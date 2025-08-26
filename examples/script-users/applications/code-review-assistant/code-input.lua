-- Code Input for Code Review Assistant
-- This file provides code samples to be reviewed
-- Similar to how webapp-creator uses user-input-ecommerce.lua

return {
    -- Testing with just one file for faster execution
    {
        filename = "auth.js",
        language = "javascript",
        code = [[
// User authentication module
const bcrypt = require('bcrypt');
const jwt = require('jsonwebtoken');

class AuthService {
    constructor() {
        this.users = {};
        this.SECRET_KEY = 'hardcoded-secret-key-123'; // SECURITY: Hardcoded secret
    }
    
    async login(username, password) {
        const user = this.users[username];
        
        // SECURITY: No rate limiting
        // QUALITY: No input validation
        if (!user) {
            throw new Error('User not found');
        }
        
        // SECURITY: Using MD5 instead of bcrypt
        const crypto = require('crypto');
        const hash = crypto.createHash('md5').update(password).digest('hex');
        
        if (hash === user.passwordHash) {
            // SECURITY: JWT with no expiration
            const token = jwt.sign({ username }, this.SECRET_KEY);
            console.log(`User ${username} logged in`); // SECURITY: Logging sensitive info
            return token;
        }
        
        throw new Error('Invalid password');
    }
    
    // QUALITY: No error handling
    async registerUser(username, password) {
        // SECURITY: Weak password requirements
        if (password.length < 4) {
            throw new Error('Password too short');
        }
        
        const crypto = require('crypto');
        const hash = crypto.createHash('md5').update(password).digest('hex');
        
        // PRACTICES: Direct object mutation
        this.users[username] = { passwordHash: hash };
    }
}

module.exports = AuthService;
]]
    }
}