<div align="center" style="margin: 30px;">
     <img src="https://i.ibb.co/wzWbcpm/logo.png" alt="logo" />
     <div align="center">
        <a href="##Installation">Installation</a> |
        <a href="/">Documentation</a> | 
        <a href="https://github.com/proxin187">Author</a>
    </div>
    <br>
    <strong>A shell written with simplicity and easy use in mind.</strong>
</div>

<b>WARNING:</b> Nsh is still in early development and may have bugs, if you find a bug feel free to report it on <a href="https://twitter.com/idkfwayfuiwa">Twitter</a>.

## Description
---
The N shell is focusing on performance and simplicity while still being useful. nsh is shipped with a custom implementation of readline and auto completion.

## Roadmap
---
- [x] Easy Configuration (~/.config/nsh/conf.nsh)
- [x] Environment variable integration
- [ ] Signal handling

## Installation
---

```
$ git clone https://github.com/proxin187/Nsh
```


run installation script:

```
$ chmod +x install.sh
$ sudo ./install.sh
```

## Usage
---

```
nsh [OPTIONS]
    OPTIONS:
        -c <FILE>: load custom config
        -help: show this
```


## License
---
Nsh is licensed under the MIT license.

## Features
---
The features of nsh are listed below but these specifications are simple, if you want in depth explanation of all the features i recommend you go read the documentation.

| Name  | Usage                            |
| ---   | ---                                  |
| `Executable` | `<executable> [ARGS]` |
| `Change directory` | `cd <location>` |
| `Alias`  | `Alias <original> <replacement>` |
| `Enviroment Variable`  | `$<var>$ = <value>` |

## Contribute
---
Nsh is currently not open for contribution, this may change as Nsh gets more mature.
