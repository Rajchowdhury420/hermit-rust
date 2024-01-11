+++
title = "Listener"
date = 2024-01-11
[extra]
toc=true
+++

## Add

Hermit currently supports the `HTTPS` listener only.  
You need to specify the domains (`-d`) for HTTPS self-signed certificates.

```sh
Hermit $ listener add -d localhost,my-c2-server.com
```

Once the listener is added successfully, you can see it with the `listeners` (or `listener list`) command.

```sh
Hermit $ listeners
```

<br />

## Start

After adding a listener as the previous section, you can start it by the `listener start <ID or Name>` command.

```sh
Hermit $ listener start 1
```

<br />

## Stop

```sh
Hermit $ listener stop 1
```

<br />

## Delete

Listeners that you don't plan to use can be deleted using the `delete` command.

```sh
Hermit $ listener delete 1
```

If you want to delete all listeners, specify `all` to the second argument as below:

```sh
Hermit $ listener delete all
```