# An utterly dumb JSON Array to JSON Lines converter

Some folks give me gigantic JSON array files all in one line,
and none of the big data tools want to load them.  I used to 
convert them to JSON Lines or CSV with [`jq`](https://stedolan.github.io/jq/),
but it uses gigantic amounts of memory.  I suck at Rust but I
was nevertheless able to write this dumb utility to convert to 
JSON Lines.  It just reads from stdin and writes to stdout.