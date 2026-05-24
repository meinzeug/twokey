# Security

## Baseline

TwoKey must be conservative because it operates near global input, selected text, clipboard content, and potentially sensitive documents.

## Confirmation Required

The app must not perform these actions without explicit confirmation:

- Delete files.
- Move files.
- Execute terminal commands.
- Change system configuration.
- Send emails.
- Submit forms.

## Text Editing

The default behavior for replacing selected text should be preview first, then confirm. A later setting may allow immediate replacement for trusted workflows.

## Provider Privacy

External provider use must be visible when selected text, documents, images, or transcripts leave the machine.

API keys must be stored through a secure local mechanism where possible and never committed to source code.

## Clipboard Handling

Future clipboard-based automation must save and restore existing clipboard content. Failures must be visible to the user.
