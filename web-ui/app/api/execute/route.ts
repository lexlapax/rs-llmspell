import { NextResponse } from 'next/server';
import { spawn } from 'child_process';
import path from 'path';

export async function POST(request: Request) {
    try {
        const body = await request.json();
        const { type, name, params } = body;

        if (!type || !name) {
            return NextResponse.json({ error: 'Missing type or name' }, { status: 400 });
        }

        const binaryPath = path.resolve(process.cwd(), '../target/release/llmspell');
        const args = [type, 'exec', name];

        if (params) {
            Object.entries(params).forEach(([key, value]) => {
                if (value !== undefined && value !== null && value !== '') {
                    args.push('--param', `${key}=${value}`);
                }
            });
        }

        console.log('Executing:', binaryPath, args.join(' '));

        const child = spawn(binaryPath, args);

        const stream = new ReadableStream({
            start(controller) {
                child.stdout.on('data', (data) => {
                    controller.enqueue(data);
                });

                child.stderr.on('data', (data) => {
                    controller.enqueue(data);
                });

                child.on('close', (code) => {
                    controller.close();
                });

                child.on('error', (err) => {
                    controller.enqueue(new TextEncoder().encode(`Error: ${err.message}\n`));
                    controller.close();
                });
            },
        });

        return new NextResponse(stream);
    } catch (error) {
        console.error('Error executing command:', error);
        return NextResponse.json({ error: 'Failed to execute command' }, { status: 500 });
    }
}
