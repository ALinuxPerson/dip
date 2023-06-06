# **D**iscord **I**PC **P**roxy

[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]
[![MIT License][license-shield]][license-url]
[![Made with Rust][rust-shield]][rust-url]

> Discord Rich Presence (and other features) without having Discord on!

# Why?

I occasionally run memory intensive applications on my computer. Unfortunately, I often can't run Discord simultaneously
with these application without the risk of running out of memory. However, I still want my friends to be aware of what 
I'm currently doing or playing. Since I have a second computer it *would* be theoretically possible to redirect or proxy
all the requests sent to the IPC socket to my computer. I tried looking online to see if someone has already implemented
something akin to this project, but, alas, I've found none. Due to this, I decided to develop my own.

# Getting Started

## Prerequisites

* A second computer capable of running a Discord client (not the website).

## Instructions

1. Go to releases and download the package meant for your operating system (or compile it manually).
    * There are two binaries packaged for each operating system, the host and remote. Here is information as to which
      program is meant to be run on which computer.
     
      * **Host**: This is meant to be run on the computer which isn't running Discord. 
      * **Remote**: This is meant to be run on the computer which is running Discord. 
    
      For more information, see the terminologies section.
2. Link up your host computer and remote computer.
   * First, ensure that the two computers are on the same local area network.
   * Then, launch `dip_remote` on your remote. It should tell you the remote address (both IPv4 and IPv6). Try the IPv4
     address first, and if it doesn't work, use the IPv6 address. Make sure Discord is open!
   * Launch `dip_host` on your host, then pass the remote address onto the host. There are two ways to do this:
     * Pass it as an argument like so: `dip_host -r REMOTE_ADDRESS` or `dip_host --remote-address REMOTE_ADDRESS`.
       Here's an example:
       ```bash
       $ dip_host -r 192.168.86.31 # if the remote address uses the default port 49131, it can be elided
       $ dip_host --remote-address 192.168.86.31:20800
       ```
     * Through a configuration file. See the section below on how to configure the host and remote.
3. You're all good to go!

## Configuration

The configuration resolves in the following order: arguments passed, configuration files.

The location of the configuration file is told when the `RUST_LOG` environment variable is set to `DEBUG` on both the 
host and remote binary:

```
2023-06-06T09:02:09.085950Z DEBUG dip_common: config file location is /home/alp/.config/dip/host.toml // <-- config file location
2023-06-06T09:02:09.086546Z  INFO dip_host: socket path is /run/user/1000/discord-ipc-0
2023-06-06T09:02:09.086563Z  INFO dip_host: remote address to connect to remote_address=192.168.86.32:49131
2023-06-06T09:02:09.086577Z  INFO dip_host: successfully resolved configuration
2023-06-06T09:02:09.086661Z DEBUG dip_host::utils: destroy path /run/user/1000/discord-ipc-0 on termination
2023-06-06T09:02:09.086682Z DEBUG dip_common::serve: start serving connections
```

Here are the locations of `host.toml` and `remote.toml` depending on operating system. **Note that Windows is not 
supported yet (although it is being worked on), and macOS is untested.**

* Linux
  * **host.toml**: `$XDG_CONFIG_HOME/dip/host.toml` or `$HOME/.config/dip/host.toml`, example is 
                     `/home/alp/.config/dip/host.toml`
  * **remote.toml**: `$XDG_CONFIG_HOME/dip/remote.toml` or `$HOME/.config/dip/remote.toml`, example is
                     `/home/alp/.config/dip/remote.toml`
* Mac OS
    * **host.toml**: `$HOME/Library/Application Support/ALinuxPerson.DIP/host.toml`, example is
      `/Users/AMacPerson/Library/Application Support/ALinuxPerson.DIP/host.toml`
    * **remote.toml**: `$HOME/Library/Application Support/ALinuxPerson.DIP/remote.toml`, example is
      `/Users/AMacPerson/Library/Application Support/ALinuxPerson.DIP/remote.toml`
* Windows
    * **host.toml**: `{FOLDERID_RoamingAppData}\ALinuxPerson\DIP\config\host.toml`, example is
                     `C:\Users\AWindowsPerson\AppData\Roaming\ALinuxPerson\DIP\config\host.toml`
    * **remote.toml**: `{FOLDERID_RoamingAppData}\ALinuxPerson\DIP\config\remote.toml`, example is
                     `C:\Users\AWindowsPerson\AppData\Roaming\ALinuxPerson\DIP\config\remote.toml`

Example configurations of `host.toml` and `remote.toml` can be found here: [host.toml](host/host.toml), [remote.toml](remote/remote.toml)

# OS Support

* Linux

Linux is officially supported as the author of this project uses Linux as their daily driver. If a specific distro were
to be given more support though, it would be Arch Linux for the same reason why Linux is officially supported.

* macOS

macOS *theoretically* should be compatible, as it uses a Unix base just like Linux. However, it is untested, so use at 
your own risk.

* Windows

Windows is currently not compatible due to a fundamental difference on how Discord implements IPC on Windows compared to
Unix based platforms.

On Windows, Discord uses named pipes for IPC. On Unix based platforms however, Unix Sockets are used. There isn't any
support yet for named pipes, but it is being worked on.

# Usage

`dip_remote` should be launched first on the remote computer, then `dip_host` can be launched on the host computer.

## Host

Launch `dip_host` with remote address passed in either the arguments (through -r / --remote-address) or configuration
file. It must match the address given by `dip_remote`. 

`dip_host` will tell you what remote address will be used to connect. See this sample log output to find out where it 
is:

```
2023-06-05T13:29:26.121833Z  INFO dip_host: socket path is /run/user/1000/discord-ipc-0
2023-06-05T13:29:26.121846Z  INFO dip_host: remote address to connect to remote_address=192.168.86.32:49131 // <-- this is the remote address
2023-06-05T13:29:26.121851Z  INFO dip_host: successfully resolved configuration
```

## Remote

Launch `dip_remote`. It should not take any arguments. If you want to set a port, you can pass the -p or --port 
argument.

Upon launch, `dip_remote` should tell you the remote address which will be used by `dip_host`. It will be given in IPv4,
and IPv6 if possible. Use the IPv4 address first, then use IPv6. See this sample log output to find out where it is:

```
2023-06-05T13:32:21.774508Z  INFO dip_remote: socket path is /run/user/1000/discord-ipc-0
2023-06-05T13:32:21.774534Z  INFO dip_remote: port to listen on port=49131
2023-06-05T13:32:21.774540Z  INFO dip_remote: successfully resolved configuration
2023-06-05T13:32:21.774694Z  INFO dip_remote: remote ipv4 address is 192.168.86.32:49131 // <---------- use these two
2023-06-05T13:32:21.774744Z  INFO dip_remote: remote ipv6 address is [::ffff:192.168.86.32]:49131 // <- addresses
```

# Advanced Usage

## Logging

Both the host and remote binary use `tracing_subscriber`'s environment filter, which utilizes the `RUST_LOG` 
environment variable to base what to log on. Basic values which can be used are `TRACE`, `DEBUG`, `INFO`, `WARN`, and 
`ERROR`. For more information on the syntax, see the [`tracing-subscriber` documentation][tracing-subscriber-docs-url].

# Terminologies

**Host**: The computer which will receive all requests from the virtual socket and forward them to the remote through TCP. 
          This is the computer which doesn't have Discord open.

**Remote**: The computer which will receive all requests from the host and relay them to a real socket. Responses from
            the real socket will be sent to the host through TCP. This is the computer which has Discord open.

**Host Computer**: The computer which will run the host binary.

**Remote Computer**: The computer which will run the remote binary.

**Virtual Socket**: The fake socket created by the host binary which will forward all requests to the remote. Responses
                    coming from the remote will be sent to this socket as well.

**Real Socket**: The real Discord IPC socket created by a running Discord client. The remote connects to this socket and
                 forwards all requests from the host to this socket. Responses coming from this socket will be sent to
                 the host.

# License

Distributed under the MIT license. See [`LICENSE`](LICENSE) for more information.

[contributors-shield]: https://img.shields.io/github/contributors/ALinuxPerson/dip.svg?style=for-the-badge
[contributors-url]: https://github.com/ALinuxPerson/dip/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/ALinuxPerson/dip.svg?style=for-the-badge
[forks-url]: https://github.com/ALinuxPerson/dip/network/members
[stars-shield]: https://img.shields.io/github/stars/ALinuxPerson/dip.svg?style=for-the-badge
[stars-url]: https://github.com/ALinuxPerson/dip/stargazers
[issues-shield]: https://img.shields.io/github/issues/ALinuxPerson/dip.svg?style=for-the-badge
[issues-url]: https://github.com/ALinuxPerson/dip/issues
[license-shield]: https://img.shields.io/github/license/ALinuxPerson/dip.svg?style=for-the-badge
[license-url]: https://github.com/ALinuxPerson/dip/blob/master/LICENSE.txt
[tracing-subscriber-docs-url]: https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html#directives
[rust-shield]: https://img.shields.io/badge/made%20with%20rust-e43d3f?style=for-the-badge&logo=rust&logoColor=white
[rust-url]: https://www.rust-lang.org