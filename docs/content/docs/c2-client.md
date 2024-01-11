+++
title = "C2 Client"
date = 2024-01-02
[extra]
toc=true
+++

## Run (Connect to the C2 Server)

At first, make sure that [the C2 server](./c2-server) has already started. It's running on the port `9999` by default.  
Now run the `client` command to connect the C2 server.  

```sh
$ hermit client -H 0.0.0.0 -P 9999


        ┓┏┏┓┳┓┳┳┓┳┏┳┓
        ┣┫┣ ┣┫┃┃┃┃ ┃
        ┛┗┗┛┛┗┛ ┗┻ ┻
          C2 CLIENT
      +++++++++++++++++
      DEVELOPED BY HDKS

[+] Handshake has been completed.
[+] Connected to C2 server (ws://0.0.0.0:9999/hermit) successfully.

Hermit $
```

After connecting, the client console will start.

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