# INXS

Imagine wanting to full check the integrity of files within a filesystem.
You might have an immutable gold master you wish to deeply inspect.
INXS allows you to store a hash of the filesystem and then compare it to the current state.

### Usage

Index a file filesystem

```
inxs index --path .
```

Check the difference between the indexed version of the filesystem and the current

```
inxs check --path .
```

## Installation

```
cargo install inxs
```

### Dependencies

INXS requires `etcd` to store it's results.
The easiest way to install this on MacOS is with brew:

```
brew install etcd
```
