import { NextResponse } from 'next/server';
import { exec } from 'child_process';
import util from 'util';
import path from 'path';

const execPromise = util.promisify(exec);

export async function GET() {
  try {
    const binaryPath = path.resolve(process.cwd(), '../target/release/llmspell');
    const { stdout } = await execPromise(`${binaryPath} template list --output json`);
    return NextResponse.json(JSON.parse(stdout));
  } catch (error) {
    console.error('Error listing templates:', error);
    return NextResponse.json({ error: 'Failed to list templates' }, { status: 500 });
  }
}
