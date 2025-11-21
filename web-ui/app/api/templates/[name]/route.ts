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
        const { stdout } = await execPromise(`${binaryPath} template schema ${name} --output json`);
        return NextResponse.json(JSON.parse(stdout));
    } catch (error) {
        console.error('Error getting schema:', error);
        return NextResponse.json({ error: 'Failed to get schema' }, { status: 500 });
    }
}
