# Workspace & Assertion Panels

This lesson verifies the UI panels that make environment engine exercises visible to students.

## Workspace Panel

When an exercise has an `environment:` block with files, directories, symlinks, services, or ports, a **WORKSPACE** panel appears between the exercise prompt and the code box. It shows what the engine set up before your code runs.

Look for these icons in the workspace panel:
- **📄** Files (with content preview, truncated if long)
- **📁** Directories
- **🔗** Symlinks (link → target)
- **🔌** Services (background processes)
- **🌐** Dynamic ports
- **⚙️** Setup steps

## Assertion Checklist

Exercises using `validation: method: state` show an **EXPECTED** panel with `○` (unchecked) items before you run. After submitting, the panel changes to **RESULTS** with `✔` (pass) or `✘` (fail) per assertion.

This gives immediate visual feedback on exactly what passed and what didn't — no need to parse error messages.
