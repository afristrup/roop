pub const USAGE: &str =
    "usage: roop build <input.roop> [-o <output>] [--emit ir|obj|exe] [--link <file>]...
       roop lean <input.roop> [-o <output.lean>] [--check]
       roop fmt [<path>...] [--check] [--width <columns>] [--stdin]

  build        compile to LLVM IR, an object, or an executable
    --emit ir    write LLVM IR (and AIR/PTX modules) for inspection
    --emit obj   write a relocatable object (default)
    --emit exe   link an executable; --link supplies the main (a .c/.ll/.o file)
    --link FILE  extra input for linking, implies --emit exe

  lean         translate to a Lean 4 model with reversibility theorems
    --check      run Lean on the result and fail if it is rejected

  fmt          rewrite .roop files in place, like cargo fmt
    <path>       files or directories; by default every .roop file of the project
    --check      change nothing, list the files that would change, and fail if any
    --width N    line width, overriding max_width of [format]
    --stdin      format the text on stdin and write it to stdout

Settings come from the nearest Roop.toml ([parallel] controls target choice,
[format] sets max_width and indent).";
