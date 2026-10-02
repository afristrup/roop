pub const USAGE: &str =
    "usage: roop build <input.roop> [-o <output>] [--emit ir|obj|exe] [--link <file>]...

  --emit ir    write LLVM IR (and AIR/PTX modules) for inspection
  --emit obj   write a relocatable object (default)
  --emit exe   link an executable; --link supplies the main (a .c/.ll/.o file)
  --link FILE  extra input for linking, implies --emit exe

Settings come from the nearest Roop.toml ([parallel] controls target choice).";
