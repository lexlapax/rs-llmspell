
const verifyOutput = async () => {
    console.log("Connecting to WebSocket...");
    const ws = new WebSocket('ws://localhost:3000/ws/stream');

    await new Promise((resolve) => ws.onopen = resolve);
    console.log("WebSocket connected. Triggering script...");

    // Listen for messages
    const messagePromise = new Promise((resolve, reject) => {
        const timeout = setTimeout(() => reject(new Error("Timeout waiting for output")), 5000);
        ws.onmessage = (event) => {
            const data = JSON.parse(event.data);
            console.log(`Received event: ${data.event_type}`);
            if (data.event_type === 'kernel.iopub.stream') {
                const text = data.data?.content?.text || '';
                console.log(`Stream content: "${text}"`);
                if (text.includes('TEST_VERIFICATION_OUTPUT')) {
                    clearTimeout(timeout);
                    resolve(true);
                }
            }
        };
    });

    // Trigger script
    const response = await fetch('http://localhost:3000/api/scripts/execute', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            code: "print('TEST_VERIFICATION_OUTPUT')",
            engine: "lua"
        })
    });

    console.log(`Execute response: ${response.status}`);

    try {
        await messagePromise;
        console.log("SUCCESS: Found expected output in stream!");
        process.exit(0);
    } catch (e) {
        console.error(e.message);
        process.exit(1);
    }
};

verifyOutput();
