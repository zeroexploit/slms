# Simple Linux Media Server
The following Lines will describe what this Project is about and what targets should be reached with the final Application. As for now, this project is under high development and there might be changes not descriptive in here. As there is no actual working Version of SLMS, make sure to always be informed on what is currently going on here!

## Target
The main target of the SLMS Project is to provide a full-featured UPnP Media Server that is capable of providing any form of Media File to a Network Device known as Renderer. And while doing so, it should use as least system resources as possible. To achieve this, every media that can be directly exposed to a renderer should be served as it is. Everything a Renderer can not handle should be transcoded in realtime to make it available as well. In order to make transcoding as efficient as possible only the medias tracks that need to be converted should be transcoded. For example: If a TV can handle a x264 video stream, it should get that one directly. But if it may not be able to handle dts audio, the audio track needs to be converted to (for example) ac3 on the fly without touching the Video Track. And in order to satisfy as many renderers as possible, the servers configuration should depend on the Renderer itself instead of the server. That means it should be possible to configure Renderers individually, resulting in a Media Server providing independent configurations for whatever type of client is asking for a media file.
After providing a full featured UPnP Media Server, additional DLNA Content should be able to be provided. Meaning that a full DLNA implementation should occur. Regarding the main target, even this should be able to be configured for each Renderer individually.
Once that is done, this Application should be a fully featured UPnP AND DLNA MediaServer. Including all tasks a Renderer might send to create new Structures inside the Media Database System. For Example: Playlists, Genres, Recordings, etc.
As this is a long way to go, the SLMS Project will concentrate on extending its possibilities in every Version starting from the root of providing simple Files as they are, to the point everything explains in UPnP MediaServer and DLNA Standard is possible.

## Restrictions
If something can be written by ourselves, it should be done. As this Application should run on hardware as small as possible, we do not want to include some big fat libraries we do not have control of. Everything used here should be as efficient as possible. That means: Every Protocol that needs to be implemented should be concentrated down to the features required here only! To make it clear: Do not use any external Library if you do not need all of its features. If you need some functionality, add it yourself!
Well, of course there is one library we need to use in order to keep everything nice and clean: the ffmpeg library to handly media files. Doing all that stuff on our own would definitely way to much! 
All in all, make sure to avoid any external tools if you can code it on your own!

## Way To Go
There are a lot of modules that need to be written in Order to SLMS become one of the most useful Media Servers:

* `Logging` - Module for creating consistent and thread safe Logging. Required for every other module to output Information.
* `Sockets` - Implementation of Sockets to allow communication over a Network
* `SSDP` - Announce the Media Server in the Local Network and make it available for Renderer Devices using SSD Protocol.
* `UPnP` - UPnP Protocol implementation to communicate with network devices, including Connection Manager, Content Directory and Media Server.
* `HTTP` - Allow Communication over the HTTP Protocol including streaming. Needs for a lot of other Sub - Implementations.
* `RTP` - Implementation of the Real Time Streaming Protocol in Order to serve renderers more advanced than simple HTTP.
* `DLNA` - Allows DLNA Devices to interact with the Media Server. Needs a lot of Sub - Implementations.
* `Transcoding` - Allow real-time Encoding of unsupported Content in Order to make it available to a Renderer.
* `Meta-Data` - Module to gain additional Meta Data for a Media Item from online Resources.
* `Content-Containers` - Allow to create Playlist, Genres and other Collections of Media Items independent of the real File System structure.

These Modules are required to gain the projects main target. But as they are listed here, most of them will require a lot of sub modules, therefore this list can never be more than a simple overview of what needs to be done. If anyone feels the need to provide one of these modules, make sure to implement them well!
