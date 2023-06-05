# **D**iscord **I**PC **P**roxy
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

1. Go to releases and download the package meant for your operating system.
    * There are two binaries packaged for each operating system, the host and remote. Here are explanations for what
      they mean:
      * **Host**: This is meant to be run on the computer which isn't running Discord. It will forward all requests to 
                  the remote, and all responses from the remote to the socket.
      * **Remote**: This is meant to be run on the computer which is running Discord. It will forward all requests from
                    the host to the Discord IPC, and all responses from Discord IPC to the host.
2. Link up your host and remote.
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

Here are the locations of `host.toml` and `remote.toml` depending on operating system.

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

# Usage

## Host

## Remote

# License

Distributed under the MIT license. See [`LICENSE`](LICENSE) for more information.