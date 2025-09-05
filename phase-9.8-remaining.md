
  1. Core CLI Functionality:
   * Script Arguments: The CLI can parse arguments for a script (e.g., llmspell run 
     script.lua -- arg1), but these arguments are not actually passed to the script itself.
     This is identified as an "architectural gap".
   * Output Formatting: The --format flag (e.g., --format json) is not implemented for the
     run command.
   * Engine Selection: The --engine flag for the run command is not implemented, so you
     cannot select a script engine (like js) from the command line.

  2. State and Session Persistence:
   * Session Persistence is Critically Broken: This is the most significant issue found.
     Even when state persistence is enabled in the configuration, the state object is not
     available in scripts. This means the kernel is not loading or injecting the persisted
     state, making stateful applications non-functional.
   * Multi-Client Sessions: The architecture does not support multiple CLI instances
     connecting to the same kernel session. The --connect flag required for this is not
     implemented.

  3. Debugging:
   * `.locals` REPL Command: The debug command to inspect local variables is not implemented
     and times out when used.
   * No Standalone Debug Command: There is no llmspell debug <script> command to launch a
     script directly in a debugging session.
   * No DAP Support: There is no integration with the Debug Adapter Protocol (DAP) for use
     with IDEs.

