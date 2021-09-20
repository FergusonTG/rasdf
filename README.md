# RASDF : a fasd clone written in Rust.

I wrote this as an exercise to learn the Rust programming language; it
also struck me that fasd itself has not been maintained for some
years. Rust is a high-level language that produces fast executables,
and since the program runs as a PROMPT_COMMAND function, it needs to
be as quick as possible. 

Please bear in mind that this is my first excursion into Rust, so
there are certainly some horrible bits of code. If you let me know
kindly, I can make it better and learn at the same time. 

## Command line

rasdf [OPTIONS] {init,clean,add,remove,find,find-all} ARGUMENTS

### Options: 
  -a	Any type of result: file or folder
  -d    Folders (directories) only
  -f    Files only

  -i	case-insensitive
  -c    case-sensitive

  -s    strict (last argument must match last segment of path)
  -l    lax (strict does not apply)

  -D    scoring method Date
  -F    scoring method Frecency (default)
  -R    scoring method Rating

### Commands:
  init:  create a new empty database

  clean: compare data base to RASDF_MAXROWS; if it is over-long,
  remove least-used files according to frecency and age all ratings.

  add:   add one or more rows to the database. Each argument must be a
  valid and existing path.

  remove: remove one row from the database. No error is raised if the
  row was not previously in the database. 

  find:   print one row if there is a match to the arguments; or
  nothing in the case of no match. The arguments are read literally
  and matched against each path in order; but do not have to match
  different segments (eg find ‘my ile tx’ will match myfile.txt). The
  result is printed on stdout, suitable for $( rasdf find ... )
  substitution.

  find-all: print paths and scores on one line each for all matches;
  matching is carried out as described above.

### Arguments

For add and remove, the arguments are valid and existing file paths;
they can be given as absolute or relative paths and will be
canonicalized. 

For find and find-all, the arguments are items to match. If you need
to match a space, try enclosing it in apostrophes. 

## Environment variables

There is no rc file for configuration; you can preset options by using
environment variables.

  RASDF_DATAFILE
    Absolute path to the desired datafile.
    Default $HOME/.config/rasdf/rasdf.dat

  RASDF_FLAGS
    Set any of the command-line switches in the same way. If flags are
    repeated or contradictory, the latest version wins: command-line
    flags are read after the RASDF_FLAGS.

  RASDF_METHOD
    Scoring method: one of {date, rating, frecency}.
    Default frecency

  RASDF_MAXLINES
    Maximum number of lines in the database file. 
    Default 200

  RASDF_LOGFILE
    Absolute path of logging file (not implemented yet).


