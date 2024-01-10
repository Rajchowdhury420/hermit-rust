+++
title = "C2 Server"
date = 2024-01-02
[extra]
toc=true
+++

## Start C2 Server

```sh
$ hermit server


        ┓┏┏┓┳┓┳┳┓┳┏┳┓
        ┣┫┣ ┣┫┃┃┃┃ ┃
        ┛┗┗┛┛┗┛ ┗┻ ┻
          C2 SERVER
      +++++++++++++++++
      DEVELOPED BY HDKS

[2024-01-10T13:59:41Z INFO  hermit::server::certs::https] /home/ubuntu/.hermit/server/listeners/listener_3943635548/certs/cert.pem created successfully.
[2024-01-10T13:59:41Z INFO  hermit::server::certs::https] /home/ubuntu/.hermit/server/listeners/listener_3943635548/certs/key.pem created successfully.
[2024-01-10T13:59:41Z WARN  hermit::server::db::listeners] Listener already exists in database.
[2024-01-10T13:59:41Z INFO  hermit::server::server] listening on 0.0.0.0:9999
```

The C2 server will start on `0.0.0.0:9999` by default.