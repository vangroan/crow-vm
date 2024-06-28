
# Language Feature Tests

These are the end-to-end tests for the language features.

<dl>
    <dt>core</dt>
    <dd>
        Tests for the core module. These are native functions that are bound
        to every module implicitly when it's initialised. The VM calls these
        functions similarly to user defined native functions.
    </dd>
    <dd>
        They are distinct from <i>built-ins</i> which are implemented inline
        in the interpreter loop itself.
    </dd>
    <dt>local</dt>
    <dd>
        Tests for local variables.
    </dd>
</dl>
