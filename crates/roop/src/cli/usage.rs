pub const USAGE: &str =
    "usage: roop build <input.roop> [-o <output>] [--emit ir|obj|exe] [--link <file>]...
       roop run <input.roop> [<args>...]
       roop lean <input.roop> [-o <output.lean>] [--check]
       roop test [<path>...] [--filter <text>] [--timeout <seconds>] [--lean]
       roop fmt [<path>...] [--check] [--width <columns>] [--stdin]
       roop weave <model.json> [-o <output.roop>] [--tests] [--batch <samples>]
                  [--driver <main.c>]
       roop lsp

  build        compile to LLVM IR, an object, or an executable
    --emit ir    write LLVM IR (and AIR/PTX modules) for inspection
    --emit obj   write a relocatable object (default without a `main`)
    --emit exe   link an executable; --link supplies the main (a .c/.ll/.o file)
    --link FILE  extra input for linking, implies --emit exe

  run          build a program with a `main` and run it; the arguments after the
               input are the program's, and its exit status is the result

  lean         translate to a Lean 4 model with reversibility theorems
    --check      run Lean on the result and fail if it is rejected

  test         run the `test` items of .roop files, each forward and then backward
    <path>       files or directories; by default every file of the project with tests
    --filter T   only the tests whose name contains T
    --timeout S  seconds a test may run (default 60)
    --lean       run each test on the Lean model too and compare

  fmt          rewrite .roop files in place, like cargo fmt
    <path>       files or directories; by default every .roop file of the project
    --check      change nothing, list the files that would change, and fail if any
    --width N    line width, overriding max_width of [format]
    --stdin      format the text on stdin and write it to stdout

  weave        compile a model of reversible layers (JSON, from
               crates/roop-weave/python/torch_to_weave.py) to weave code
    --tests      add a test that checks the code against a reference in doubles
    --batch N    add `<name>_train`, a training step of N samples that C can call
    --driver F   write a C program that trains the model on data files (needs --batch)

  lsp          language server on stdin and stdout: syntax and check errors as
               diagnostics, formatting, and an outline of the file

Settings come from the nearest Roop.toml ([parallel] controls target choice,
[format] sets max_width and indent).";
