import { NextResponse } from 'next/server';
import { exec } from 'child_process';
import util from 'util';
import path from 'path';

const execPromise = util.promisify(exec);

export async function GET(
    request: Request,
    { params }: { params: Promise<{ name: string }> }
) {
    try {
        const { name } = await params;
        const binaryPath = path.resolve(process.cwd(), '../target/release/llmspell');
        const { stdout } = await execPromise(`${binaryPath} template info "${name}" --show-schema --output json`);
        const data = JSON.parse(stdout);

        // Map the response to match TemplateSchema interface
        const schema = {
            name: data.name,
            description: data.description,
            params: data.schema?.parameters || [],
            version: data.schema?.version || data.version
        };

        return NextResponse.json(schema);
    } catch (error) {
        console.error('Error getting schema:', error);
        return NextResponse.json({ error: 'Failed to get schema' }, { status: 500 });
    }
}
