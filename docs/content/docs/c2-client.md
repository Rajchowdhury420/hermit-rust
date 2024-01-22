+++
title = "C2 Client"
date = 2024-01-02
[extra]
toc=true
+++

## Run (Connect to the C2 Server)

Make sure that [the C2 server](./c2-server) has already started first.  
Run the `client` command to connect the C2 server.  

```sh
$ hermit client -P 9999


        ┓┏┏┓┳┓┳┳┓┳┏┳┓
        ┣┫┣ ┣┫┃┃┃┃ ┃
        ┛┗┗┛┛┗┛ ┗┻ ┻
          C2 CLIENT
      +++++++++++++++++
      DEVELOPED BY HDKS

[+] Connected the C2 server (http://[::1]:9999) successfully.

Hermit $
```

<br />

## Help

```sh
Hermit $ help
```

<br />

## Listener

See [Listener](/docs/listener) for more details.

<br />

## Implant

See [Implant](/docs/implant) for more details.

<br />

## Agent

See [Agent](/docs/agent) for more details.

<br />

## Exit

To quit the client console, run the `exit` command.

```sh
Hermit $ exit
```