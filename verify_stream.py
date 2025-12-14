import asyncio
import aiohttp
import json
import sys

async def verify_output():
    print("Connecting to WebSocket...")
    async with aiohttp.ClientSession() as session:
        async with session.ws_connect('http://localhost:3000/ws/stream') as ws:
            print("WebSocket connected. Triggering script...")
            
            # Trigger script execution
            script_payload = {
                "code": "print('TEST_VERIFICATION_OUTPUT')",
                "engine": "lua"
            }
            
            async with session.post('http://localhost:3000/api/scripts/execute', json=script_payload) as resp:
                print(f"Execute response: {resp.status}")
                if resp.status != 200:
                    print(await resp.text())
                    return False

            print("Waiting for stream message...")
            try:
                # Wait for messages
                while True:
                    msg = await asyncio.wait_for(ws.receive(), timeout=5.0)
                    if msg.type == aiohttp.WSMsgType.TEXT:
                        data = json.loads(msg.data)
                        print(f"Received event: {data.get('event_type')}")
                        
                        # Check for stream content
                        if data.get('event_type') == 'kernel.iopub.stream':
                            content = data.get('data', {}).get('content', {})
                            text = content.get('text', '')
                            print(f"Stream content: {repr(text)}")
                            if 'TEST_VERIFICATION_OUTPUT' in text:
                                print("SUCCESS: Found expected output in stream!")
                                return True
                    elif msg.type == aiohttp.WSMsgType.CLOSED:
                        print("WebSocket closed")
                        break
                    elif msg.type == aiohttp.WSMsgType.ERROR:
                        print("WebSocket error")
                        break
            except asyncio.TimeoutError:
                print("Timed out waiting for output")
                return False

if __name__ == "__main__":
    try:
        if asyncio.run(verify_output()):
            sys.exit(0)
        else:
            sys.exit(1)
    except Exception as e:
        print(f"Error: {e}")
        sys.exit(1)
