const { exec } = require('child_process');
const util = require('util');
const path = require('path');

const execPromise = util.promisify(exec);

async function verifySchemaMapping(name) {
    try {
        const binaryPath = path.resolve(process.cwd(), '../target/release/llmspell');
        console.log(`Testing with name: "${name}"`);

        // Simulate the API route logic
        const command = `${binaryPath} template info "${name}" --show-schema --output json`;
        const { stdout } = await execPromise(command);
        const data = JSON.parse(stdout);

        // Map the response
        const schema = {
            name: data.name,
            description: data.description,
            params: data.schema?.parameters || [],
            version: data.schema?.version || data.version
        };

        console.log('Mapped Schema Keys:', Object.keys(schema));
        console.log('Params found:', Array.isArray(schema.params) ? schema.params.length : 'Not an array');

        if (!schema.name || !schema.description || !Array.isArray(schema.params)) {
            console.error('FAILED: Missing required fields');
            return false;
        }

        console.log('SUCCESS: Schema mapped correctly');
        return true;
    } catch (error) {
        console.error('Error:', error.message);
        return false;
    }
}

(async () => {
    await verifySchemaMapping('research-assistant');
})();
