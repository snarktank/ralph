#!/usr/bin/env python3
"""
OpenRouter API caller for Ralph loop.

Sends the prompt to an OpenRouter model and streams the response.
Requires OPENROUTER_API_KEY environment variable or .env file.

Usage:
  python openrouter_call.py <prompt_file> [--model MODEL] [--temperature T]
  echo "prompt" | python openrouter_call.py - [--model MODEL]
"""
import io
import json
import os
import sys
import urllib.request

sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8', errors='replace')
sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding='utf-8', errors='replace')


def load_api_key():
    """Load OpenRouter API key from environment or .env file."""
    key = os.environ.get('OPENROUTER_API_KEY', '')
    if key:
        return key
    # Try .env file
    for p in ['.env', os.path.join(os.path.dirname(__file__), '.env')]:
        try:
            with open(p, 'r') as f:
                for line in f:
                    line = line.strip()
                    if line.startswith('OPENROUTER_API_KEY='):
                        return line.split('=', 1)[1].strip().strip('"').strip("'")
        except FileNotFoundError:
            continue
    return ''


def main():
    import argparse
    parser = argparse.ArgumentParser(description='Call OpenRouter API with a prompt file')
    parser.add_argument('prompt_file', help='Path to prompt file, or - for stdin')
    parser.add_argument('--model', default='nvidia/llama-3.1-nemotron-ultra-253b:free',
                        help='OpenRouter model ID (default: nvidia/llama-3.1-nemotron-ultra-253b:free)')
    parser.add_argument('--temperature', type=float, default=0.7, help='Temperature (default: 0.7)')
    parser.add_argument('--max-tokens', type=int, default=16384, help='Max tokens (default: 16384)')
    parser.add_argument('--system-prompt', default=None, help='Optional system prompt file')
    args = parser.parse_args()

    api_key = load_api_key()
    if not api_key:
        print("ERROR: No OPENROUTER_API_KEY found. Set it in environment or .env file.", file=sys.stderr)
        sys.exit(1)

    # Read prompt
    if args.prompt_file == '-':
        prompt = sys.stdin.read()
    else:
        with open(args.prompt_file, 'r', encoding='utf-8') as f:
            prompt = f.read()

    messages = []

    # Optional system prompt
    if args.system_prompt:
        with open(args.system_prompt, 'r', encoding='utf-8') as f:
            messages.append({'role': 'system', 'content': f.read()})

    messages.append({'role': 'user', 'content': prompt})

    payload = json.dumps({
        'model': args.model,
        'messages': messages,
        'temperature': args.temperature,
        'max_tokens': args.max_tokens,
    }).encode()

    req = urllib.request.Request(
        'https://openrouter.ai/api/v1/chat/completions',
        data=payload,
        headers={
            'Content-Type': 'application/json',
            'Authorization': f'Bearer {api_key}',
        }
    )

    try:
        resp = urllib.request.urlopen(req, timeout=600)
        data = json.loads(resp.read())
        content = data.get('choices', [{}])[0].get('message', {}).get('content', '')
        print(content)
    except urllib.error.HTTPError as e:
        body = e.read().decode('utf-8', errors='replace')
        print(f'ERROR: HTTP {e.code}: {body}', file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f'ERROR: {e}', file=sys.stderr)
        sys.exit(1)


if __name__ == '__main__':
    main()
