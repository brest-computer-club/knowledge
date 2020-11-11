# Knowledge 

Knowledge let's you transform your markdown files into a knowledge base

## Installation

- Download a release
- Make it executable
- Launch it in any folder containing your notes

Your (default) web browser will open and let you browse your files by tags

## Expected markdown format

A yaml header containing at least a title and a tag is required :

```
---
title: Knowledge transform your markdown files into a knowledge base 
tags:
- rust
- elm
- markdown
- GPL licence
---

# My markdown content 
...

```

## Supported features
- [x] walk all sub-directories and discover all well-formatted files
- [x] handle relative links between "articles"
- [x] handle local and distant images 

## TODO
- [ ] refactor both front & back ( current code quality : hackathon )
- [ ] add circle-ci
- [ ] more complex tag combination search
- [ ] use a random port to avoid conflicts 


