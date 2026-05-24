# Foundations

## Vision

TwoKey Linux AI Assistant is a small desktop assistant for speaking or working with AI without opening a browser, chat tab, or separate editor. The user should hold two keys, speak, release, and receive the result in the current desktop context.

## Core Principle: AI Without Context Switching

The assistant should live at the edge of the desktop as a compact overlay. It should help inside the user's current application rather than pulling the user into a large chat interface.

## Mode Concept

- Conversation: ask a question, optionally using selected text or files as context.
- Edit Text: read selected text, transform it with a spoken instruction, then preview or replace it.
- Dictation: transcribe speech and insert text at the cursor.
- Feedback: capture local product feedback instead of treating every utterance as a general AI request.
- File Context: attach PDFs, images, screenshots, text, or Markdown to the current AI context.

## Privacy Principles

- Prefer local processing when possible.
- Make external API use visible before sensitive data leaves the machine.
- Store configuration and data in XDG-compliant locations.
- Never hard-code API keys.
- Keep an audit trail for provider/model use and errors.

## Local/Online Hybrid Strategy

TwoKey should support Ollama and local speech/TTS tools for private workflows, while allowing OpenAI-compatible and OpenRouter-compatible providers for tasks that need stronger online models or vision support.

Routing decisions must be explicit and inspectable. Sensitive text editing should prefer local models by default.

## UX Principles

- Minimal overlay, not a chat application.
- Clear status states: ready, listening, transcribing, thinking, writing, error.
- Direct recovery when Linux desktop permissions or Wayland restrictions block an action.
- User confirmation before destructive or risky operations.
- Fast mode switching through hotkeys and the overlay menu.

## Non-Goals

- Phase 1 does not implement real AI, global hotkeys, audio capture, STT, TTS, text injection, or file parsing.
- The app will not silently bypass Wayland security restrictions.
- The app will not execute shell commands, send emails, modify files, or submit forms without explicit confirmation.
