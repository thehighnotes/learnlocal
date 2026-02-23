# Setup & Teardown

This lesson tests the Environment Engine v2 setup and teardown phases.

## Setup Commands

Setup commands run **before** student code is written to the sandbox.
They can create files, initialize databases, or prepare any state the
exercise needs. The `{dir}` placeholder resolves to the sandbox directory.

## Stdin and Teardown

Setup commands can receive stdin input. Teardown commands run **after**
student code executes and can capture their stdout to a file via
`capture_to`. State validation then checks the sandbox filesystem.
