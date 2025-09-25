//! Lua completion provider for REPL and IDE support
//!
//! Provides intelligent code completion for Lua scripts including:
//! - Global variables and functions
//! - Table members and methods
//! - Local variables in scope
//! - Language keywords

use crate::engine::bridge::{CompletionCandidate, CompletionContext, CompletionKind};
use mlua::{Lua, Value};
use std::sync::RwLock;
use std::time::{Duration, Instant};
use tracing::{debug, trace};

/// Cache for global symbols to improve performance
struct SymbolCache {
    globals: Vec<CompletionCandidate>,
    keywords: Vec<CompletionCandidate>,
    last_update: Instant,
    cache_duration: Duration,
}

impl Default for SymbolCache {
    fn default() -> Self {
        Self {
            globals: Vec::new(),
            keywords: Self::lua_keywords(),
            last_update: Instant::now(),
            cache_duration: Duration::from_secs(5), // Cache for 5 seconds
        }
    }
}

impl SymbolCache {
    fn is_valid(&self) -> bool {
        self.last_update.elapsed() < self.cache_duration
    }

    fn lua_keywords() -> Vec<CompletionCandidate> {
        vec![
            "and", "break", "do", "else", "elseif", "end", "false", "for", "function", "if", "in",
            "local", "nil", "not", "or", "repeat", "return", "then", "true", "until", "while",
        ]
        .into_iter()
        .map(|kw| CompletionCandidate {
            text: kw.to_string(),
            kind: CompletionKind::Keyword,
            signature: None,
            documentation: None,
        })
        .collect()
    }
}

/// Provides completion support for Lua scripts
pub struct LuaCompletionProvider {
    cache: RwLock<SymbolCache>,
}

impl LuaCompletionProvider {
    /// Create a new Lua completion provider
    #[must_use]
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(SymbolCache::default()),
        }
    }

    /// Get completion candidates for the given context
    pub fn get_completions(
        &self,
        lua: &Lua,
        context: &CompletionContext,
    ) -> Vec<CompletionCandidate> {
        trace!("Getting completions for: {:?}", context);

        // Check for member access (table.xxx)
        if let Some(object_name) = context.is_member_access() {
            return self.get_table_members(lua, &object_name, &context.word);
        }

        // Check for method call (string:xxx)
        if let Some(object_name) = context.is_method_call() {
            return self.get_object_methods(lua, &object_name, &context.word);
        }

        // Check for local keyword
        if context.line.trim_start().starts_with("local ") && context.word_start > 5 {
            // After "local ", suggest variable names or keywords
            return Self::filter_candidates(&self.get_keywords(), &context.word);
        }

        // Check if we're inside function arguments
        if context.is_inside_function_args() {
            return self.get_function_argument_completions(lua, context);
        }

        // Default: get global completions with keywords
        self.get_default_completions(lua, context)
    }

    /// Get completions for function arguments
    fn get_function_argument_completions(
        &self,
        lua: &Lua,
        context: &CompletionContext,
    ) -> Vec<CompletionCandidate> {
        debug!("Inside function arguments, providing global completions");
        let mut candidates = self.get_global_symbols(lua, &context.word);

        // Add useful keywords that might be used as arguments
        let arg_keywords = vec!["true", "false", "nil"];
        for kw in arg_keywords {
            if kw.starts_with(&context.word) {
                candidates.push(CompletionCandidate {
                    text: kw.to_string(),
                    kind: CompletionKind::Keyword,
                    signature: None,
                    documentation: None,
                });
            }
        }

        // If there's a function context, we could provide parameter hints
        if let Some(func_name) = context.get_function_context() {
            trace!("Function context: {}", func_name);
            // Could add function-specific parameter hints here in the future
        }

        candidates
    }

    /// Get default completions (globals and keywords)
    fn get_default_completions(
        &self,
        lua: &Lua,
        context: &CompletionContext,
    ) -> Vec<CompletionCandidate> {
        let mut candidates = self.get_global_symbols(lua, &context.word);

        // Add keywords if appropriate (at line start or after whitespace)
        if context.word_start == 0 || context.line[..context.word_start].trim().is_empty() {
            candidates.extend(Self::filter_candidates(&self.get_keywords(), &context.word));
        }

        candidates
    }

    /// Get all global symbols
    ///
    /// # Panics
    ///
    /// Panics if the cache lock is poisoned
    pub fn get_global_symbols(&self, lua: &Lua, prefix: &str) -> Vec<CompletionCandidate> {
        // Check cache first
        {
            let cache = self.cache.read().unwrap();
            if cache.is_valid() && !cache.globals.is_empty() {
                debug!("Using cached global symbols");
                return Self::filter_candidates(&cache.globals, prefix);
            }
        }

        debug!("Refreshing global symbols cache");
        let mut candidates = Vec::new();

        // Iterate through _G table
        let globals = lua.globals();
        let pairs = globals.pairs::<String, Value>();
        for pair in pairs.flatten() {
            let (name, value) = pair;

            // Skip internal/private symbols
            if name.starts_with('_') && name != "_G" && name != "_VERSION" {
                continue;
            }

            let kind = match value {
                Value::Function(_) => CompletionKind::Function,
                Value::Table(_) => CompletionKind::Module,
                _ => CompletionKind::Variable,
            };

            // Add signature for known functions
            let signature = if kind == CompletionKind::Function {
                Self::get_function_signature(&name)
            } else {
                None
            };

            candidates.push(CompletionCandidate {
                text: name,
                kind,
                signature,
                documentation: None,
            });
        }

        // Update cache
        {
            let mut cache = self.cache.write().unwrap();
            cache.globals.clone_from(&candidates);
            cache.last_update = Instant::now();
        }

        Self::filter_candidates(&candidates, prefix)
    }

    /// Get members of a table
    pub fn get_table_members(
        &self,
        lua: &Lua,
        table_name: &str,
        prefix: &str,
    ) -> Vec<CompletionCandidate> {
        let mut candidates = Vec::new();

        // Get the table from globals
        let globals = lua.globals();
        if let Ok(Value::Table(table)) = globals.get::<_, Value>(table_name) {
            // Iterate through table members
            let pairs = table.pairs::<Value, Value>();
            for pair in pairs.flatten() {
                let (key, value) = pair;

                if let Value::String(key_str) = key {
                    let name = key_str.to_str().unwrap_or_default();

                    // Skip private members
                    if name.starts_with('_') {
                        continue;
                    }

                    let kind = match value {
                        Value::Function(_) => CompletionKind::Method,
                        _ => CompletionKind::Property,
                    };

                    // Special signatures for known modules
                    let signature = if kind == CompletionKind::Method {
                        Self::get_method_signature(table_name, name)
                    } else {
                        None
                    };

                    candidates.push(CompletionCandidate {
                        text: name.to_string(),
                        kind,
                        signature,
                        documentation: None,
                    });
                }
            }
        }

        Self::filter_candidates(&candidates, prefix)
    }

    /// Get methods for an object (using metatables)
    pub fn get_object_methods(
        &self,
        lua: &Lua,
        _object_name: &str,
        prefix: &str,
    ) -> Vec<CompletionCandidate> {
        let mut candidates = Vec::new();

        // For string methods, we can get them from the string metatable
        let globals = lua.globals();
        if let Ok(Value::Table(string_lib)) = globals.get::<_, Value>("string") {
            let pairs = string_lib.pairs::<String, Value>();
            for pair in pairs.flatten() {
                let (name, _value) = pair;

                candidates.push(CompletionCandidate {
                    text: name.clone(),
                    kind: CompletionKind::Method,
                    signature: Self::get_method_signature("string", &name),
                    documentation: None,
                });
            }
        }

        Self::filter_candidates(&candidates, prefix)
    }

    /// Get Lua keywords
    fn get_keywords(&self) -> Vec<CompletionCandidate> {
        self.cache.read().unwrap().keywords.clone()
    }

    /// Filter candidates by prefix
    fn filter_candidates(
        candidates: &[CompletionCandidate],
        prefix: &str,
    ) -> Vec<CompletionCandidate> {
        if prefix.is_empty() {
            return candidates.to_vec();
        }

        candidates
            .iter()
            .filter(|c| c.text.starts_with(prefix))
            .cloned()
            .collect()
    }

    /// Get function signature for known global functions
    fn get_function_signature(name: &str) -> Option<String> {
        match name {
            "print" => Some("print(...)".to_string()),
            "error" => Some("error(message, level?)".to_string()),
            "assert" => Some("assert(v, message?)".to_string()),
            "pcall" => Some("pcall(f, ...)".to_string()),
            "xpcall" => Some("xpcall(f, msgh, ...)".to_string()),
            "require" => Some("require(modname)".to_string()),
            "loadfile" => Some("loadfile(filename, mode?, env?)".to_string()),
            "load" => Some("load(chunk, chunkname?, mode?, env?)".to_string()),
            "dofile" => Some("dofile(filename?)".to_string()),
            "type" => Some("type(v)".to_string()),
            "tostring" => Some("tostring(v)".to_string()),
            "tonumber" => Some("tonumber(e, base?)".to_string()),
            "pairs" => Some("pairs(t)".to_string()),
            "ipairs" => Some("ipairs(t)".to_string()),
            "next" => Some("next(table, index?)".to_string()),
            "select" => Some("select(index, ...)".to_string()),
            "rawget" => Some("rawget(table, index)".to_string()),
            "rawset" => Some("rawset(table, index, value)".to_string()),
            "rawequal" => Some("rawequal(v1, v2)".to_string()),
            "getmetatable" => Some("getmetatable(object)".to_string()),
            "setmetatable" => Some("setmetatable(table, metatable)".to_string()),
            "collectgarbage" => Some("collectgarbage(opt?, arg?)".to_string()),
            _ => None,
        }
    }

    /// Get method signature for known table methods
    fn get_method_signature(module: &str, method: &str) -> Option<String> {
        match module {
            "table" => match method {
                "insert" => Some("table.insert(list, pos?, value)".to_string()),
                "remove" => Some("table.remove(list, pos?)".to_string()),
                "sort" => Some("table.sort(list, comp?)".to_string()),
                "concat" => Some("table.concat(list, sep?, i?, j?)".to_string()),
                "pack" => Some("table.pack(...)".to_string()),
                "unpack" => Some("table.unpack(list, i?, j?)".to_string()),
                _ => None,
            },
            "string" => match method {
                "byte" => Some("string.byte(s, i?, j?)".to_string()),
                "char" => Some("string.char(...)".to_string()),
                "dump" => Some("string.dump(function)".to_string()),
                "find" => Some("string.find(s, pattern, init?, plain?)".to_string()),
                "format" => Some("string.format(formatstring, ...)".to_string()),
                "gmatch" => Some("string.gmatch(s, pattern)".to_string()),
                "gsub" => Some("string.gsub(s, pattern, repl, n?)".to_string()),
                "len" => Some("string.len(s)".to_string()),
                "lower" => Some("string.lower(s)".to_string()),
                "match" => Some("string.match(s, pattern, init?)".to_string()),
                "rep" => Some("string.rep(s, n, sep?)".to_string()),
                "reverse" => Some("string.reverse(s)".to_string()),
                "sub" => Some("string.sub(s, i, j?)".to_string()),
                "upper" => Some("string.upper(s)".to_string()),
                _ => None,
            },
            "math" => match method {
                "abs" => Some("math.abs(x)".to_string()),
                "acos" => Some("math.acos(x)".to_string()),
                "asin" => Some("math.asin(x)".to_string()),
                "atan" => Some("math.atan(y, x?)".to_string()),
                "ceil" => Some("math.ceil(x)".to_string()),
                "cos" => Some("math.cos(x)".to_string()),
                "deg" => Some("math.deg(x)".to_string()),
                "exp" => Some("math.exp(x)".to_string()),
                "floor" => Some("math.floor(x)".to_string()),
                "log" => Some("math.log(x, base?)".to_string()),
                "max" => Some("math.max(x, ...)".to_string()),
                "min" => Some("math.min(x, ...)".to_string()),
                "modf" => Some("math.modf(x)".to_string()),
                "pow" => Some("math.pow(x, y)".to_string()),
                "rad" => Some("math.rad(x)".to_string()),
                "random" => Some("math.random(m?, n?)".to_string()),
                "randomseed" => Some("math.randomseed(x)".to_string()),
                "sin" => Some("math.sin(x)".to_string()),
                "sqrt" => Some("math.sqrt(x)".to_string()),
                "tan" => Some("math.tan(x)".to_string()),
                _ => None,
            },
            "io" => match method {
                "close" => Some("io.close(file?)".to_string()),
                "flush" => Some("io.flush()".to_string()),
                "input" => Some("io.input(file?)".to_string()),
                "lines" => Some("io.lines(filename?)".to_string()),
                "open" => Some("io.open(filename, mode?)".to_string()),
                "output" => Some("io.output(file?)".to_string()),
                "popen" => Some("io.popen(prog, mode?)".to_string()),
                "read" => Some("io.read(...)".to_string()),
                "tmpfile" => Some("io.tmpfile()".to_string()),
                "type" => Some("io.type(obj)".to_string()),
                "write" => Some("io.write(...)".to_string()),
                _ => None,
            },
            "os" => match method {
                "clock" => Some("os.clock()".to_string()),
                "date" => Some("os.date(format?, time?)".to_string()),
                "difftime" => Some("os.difftime(t2, t1)".to_string()),
                "execute" => Some("os.execute(command?)".to_string()),
                "exit" => Some("os.exit(code?, close?)".to_string()),
                "getenv" => Some("os.getenv(varname)".to_string()),
                "remove" => Some("os.remove(filename)".to_string()),
                "rename" => Some("os.rename(oldname, newname)".to_string()),
                "setlocale" => Some("os.setlocale(locale, category?)".to_string()),
                "time" => Some("os.time(table?)".to_string()),
                "tmpname" => Some("os.tmpname()".to_string()),
                _ => None,
            },
            _ => None,
        }
    }

    /// Invalidate the cache (call after script execution)
    ///
    /// # Panics
    ///
    /// Panics if the cache lock is poisoned
    pub fn invalidate_cache(&self) {
        let mut cache = self.cache.write().unwrap();
        cache.globals.clear();
        cache.last_update = Instant::now()
            .checked_sub(cache.cache_duration)
            .unwrap_or_else(Instant::now);
    }
}

impl Default for LuaCompletionProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completion_context_parsing() {
        let ctx = CompletionContext::new("print", 5);
        assert_eq!(ctx.word, "print");
        assert_eq!(ctx.word_start, 0);

        let ctx = CompletionContext::new("table.", 6);
        assert_eq!(ctx.word, "");
        assert!(ctx.is_member_access().is_some());
        assert_eq!(ctx.is_member_access().unwrap(), "table");

        let ctx = CompletionContext::new("local x = ", 10);
        assert_eq!(ctx.word, "");
        assert_eq!(ctx.word_start, 10);

        let ctx = CompletionContext::new("str:sub", 7);
        assert_eq!(ctx.word, "sub");
        assert!(ctx.is_method_call().is_some());
    }

    #[test]
    fn test_filter_candidates() {
        let candidates = vec![
            CompletionCandidate {
                text: "print".to_string(),
                kind: CompletionKind::Function,
                signature: None,
                documentation: None,
            },
            CompletionCandidate {
                text: "pcall".to_string(),
                kind: CompletionKind::Function,
                signature: None,
                documentation: None,
            },
            CompletionCandidate {
                text: "table".to_string(),
                kind: CompletionKind::Module,
                signature: None,
                documentation: None,
            },
        ];

        let filtered = LuaCompletionProvider::filter_candidates(&candidates, "p");
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|c| c.text.starts_with('p')));

        let filtered = LuaCompletionProvider::filter_candidates(&candidates, "pr");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].text, "print");

        let filtered = LuaCompletionProvider::filter_candidates(&candidates, "");
        assert_eq!(filtered.len(), 3);
    }
}
