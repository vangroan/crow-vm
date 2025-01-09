
# Crow Design

- Top level is a definition-block, and cannot evaluate code.
  Only definition statements are allowed.
- Statements start with unambiguous keyword followed by an
  identifier.
  This allows the module to build its initial definitions
  and types without having to evaluate any functions.

```
// Each file must be explicitly part of a module.
//
// The files will be combined into a single compilation unit.
module "vangroan.example";


// Top level is a definition-block, and does not evaluate code.
// Only definition statements are allowed.


// Import other modules.
from "crow.io" import IO;


// Type declarations are unified as type expressions,
// so everything can be declared using the `type` statement.
type Vec2 = struct {
    x: Float,
    y: Float,
};

// Function pointer or closure.
type Callback = func (payload: Vec2) -> Int;

type Body = interface {};


// Function name `main` is reserved as entry point.
func main() -> int {
    IO::print("Hello, world!);

    return 0;
}

func examples() {
    let x = 1 + 2 * 3;
}

func function_pointer_example() {
    let world = Physics::World::new();
    let body1 = world.new_body();
    world.on_collision(body, func(vec2) {
        // resolve collision
        return 42;  // some info that physics need
    });
}

// Anonymous type expression in function signature.
func move(target: struct { x: Float, y: Float }) { /* ... */ }

// Polymorphic pointer (reference counted and garbage collected)
//
// The vtable might have to be built up lazily.
// The number of structs that unintentionally satisfy the
// interface could turn explode the vtable sizes.
func attack(target: &interface { health: Int }) { /* ... */ }
```

