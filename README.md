# Simple Linux Media Server
 Simple Linux Media Server - UPnP / DLNA Media Server for Linux Systems 

# Installation
 Make sure FFMpeg is installed! Perform the Installation as root!
 
	su -
	git clone https://github.com/zeroexploit/slms.git
	cd slms/res
	chmod +x ./install.sh
	./install.sh
	
 Edit the Server Configuration
 
	nano /etc/slms/server.cfg
	
 Use Command "slms" to run the Server. Add -h to display additional Options
 
	slms