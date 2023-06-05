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
      - **Host**: This is meant to be run on the computer which isn't running Discord. It will forward all requests to 
                  the remote, and all responses from the remote to the socket.
      - **Remote**: This is meant to be run on the computer which is running Discord. It will forward all requests from
                    the host to the Discord IPC, and all responses from Discord IPC to the host.
2. 