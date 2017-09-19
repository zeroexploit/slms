use std::io::prelude::*;
use std::net::TcpStream;

use configuration::ServerConfiguration;

/// # ConnectionManager
///
/// Implementation of the UPnP Connection Manager
/// required to handle UPnP Control Point Connections.
/// Provides the Device- and Services Descriptions as
/// XML formatted Strings.
///
/// # TO-DO
/// - Implement "PrepareForConnection" and "ConnectionComplete"
/// - Implement "Subscribe" and handle Events
/// - Generate ProtocollInfo from actual MimeTypes available in Media Database
/// - Actually keep track of connections
pub struct ConnectionManager<'a> {
    server_cfg: &'a ServerConfiguration,
}

impl<'a> ConnectionManager<'a> {
    /// Creates a new ConnectionManager using the given Server Configuration
    ///
    /// # Arguments
    ///
    /// * `server_cfg` - Reference to the Servers Configuration
    pub fn new(server_cfg: &'a ServerConfiguration) -> ConnectionManager<'a> {
        ConnectionManager { server_cfg }
    }

    /// Sends data to the given TCP Stream
    ///
    /// # Arguments
    ///
    /// * `response` - Message to be send
    /// * `stream` - TCP Stream to send the data to
    pub fn send_data(&self, response: &str, stream: &mut TcpStream) -> bool {

        match stream.write(response.as_bytes()) {
            Ok(_) => {
                match stream.flush() {
                    Ok(_) => true,
                    Err(_) => false,
                }
            }
            Err(_) => false,
        }
    }

    /// Uses the Tcp Stream to collect the whole Request including Header
    /// and Content.
    ///
    /// # Arguments
    ///
    /// * `stream` - Stream to read Data from
    pub fn handle_connection(&self, stream: &mut TcpStream) -> String {
        let mut buffer = [0; 65515];
        let mut content_size: usize = 0;
        let mut content_size_expected: usize = 0;
        let mut content: String = String::new();
        let mut header_received: bool = false;
        let mut header_content: String = String::new();

        loop {
            let readed = match stream.read(&mut buffer) {
                Ok(bytes) => bytes,
                Err(_) => {
                    break;
                }
            };

            let tmp_content: String = match String::from_utf8(buffer[..readed].to_vec()) {
                Ok(value) => value,
                Err(_) => break,
            };

            if header_received {
                content.push_str(&tmp_content);
                content_size += readed;

                if content_size >= content_size_expected {
                    break;
                }
            } else {
                content.push_str(&tmp_content);

                match content.find("\r\n\r\n") {
                    Some(position) => {
                        // Get Content Length
                        if position + 4 >= content.len() {
                            break;
                        }

                        let mut header = content[..position].to_lowercase();
                        header_content = content[..(position + 4)].to_string();

                        match header.find("content-length:") {
                            Some(content_pos) => {
                                if content_pos + 16 >= header.len() {
                                    break;
                                }

                                header = header[(content_pos + 16)..].to_string();
                                match header.find("\r\n") {
                                    Some(inner_pos) => {
                                        header = header[..inner_pos].to_string();
                                    }
                                    _ => {}
                                }

                                content_size_expected = match header.trim().parse::<usize>() {
                                    Ok(value) => value,
                                    Err(_) => break,
                                };
                            }
                            _ => {
                                break;
                            }
                        }

                        // Set Header is complete
                        header_received = true;

                        if position + 4 < content.len() {
                            content = content[(position + 4)..].to_string();
                            content_size = content.len();
                        } else {
                            content = String::new();
                            content_size = 0;
                        }

                        // Recheck if everything is there
                        if content_size >= content_size_expected {
                            break;
                        }
                    }
                    _ => {}
                }
            }
        }

        header_content.push_str(&content);
        return header_content;
    }

    /// Takes a Request and returns the corresponding XML Answer Content.
    /// Content only. No Headers!
    /// Returns an empty String Response could not be generated
    ///
    /// # Arguments
    ///
    /// * `request` - Request from the Renderer to process
    pub fn handle_request(&self, request: &str) -> String {
        if &request[..31] == "GET /connection/description.xml" {
            return self.get_device_description();
        } else if &request[..40] == "SUBSCRIBE /connection/connection_manager" {
            return self.do_subscribe();
        } else if &request[..38] == "GET /connection/connection_manager.xml" {
            return self.get_connection_manager_description();
        } else if &request[..37] == "GET /connection/content_directory.xml" {
            return self.get_content_directory_description();
        } else if request.find("u:GetProtocolInfo").is_some() {
            return self.get_protocoll_info();
        } else if request.find("u:PrepareForConnection").is_some() {
            // Still To-Do
        } else if request.find("u:ConnectionComplete").is_some() {
            // Still To-Do
        }

        String::new()
    }

    /// Handles the "Subscribe" Request
    fn do_subscribe(&self) -> String {
        "<e:propertyset xmlns:e=\"urn:schemas-upnp-org:event-1-0\" xmlns:s=\"urn:schemas-upnp-org:service:ConnectionManager:1\">
	        <e:property>
				<SinkProtocolInfo></SinkProtocolInfo>
		    </e:property>
		    <e:property>
			    <SourceProtocolInfo></SourceProtocolInfo>
		    </e:property>
		    <e:property>
				<CurrentConnectionIDs></CurrentConnectionIDs>
	        </e:property>
        </e:propertyset>".to_string()
    }

    /// Generates the ProtocolInfo Response
    fn get_protocoll_info(&self) -> String {
        // INCLUDE rtsp-rtp-udp:*:*:* once rtsp is supported!
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>
         <s:Envelope xmlns:s=\"http://schemas.xmlsoap.org/soap/envelope/ s:encodingStyle=\"http://schemas.xmlsoap.org/soap/encoding/\"\">
	         <s:Body>
		         <u:GetProtocolInfoResponse xmlns:u=\"urn:schemas-upnp-org:service:ConnectionManager:1\">
			         <Source>
				         http-get:*:*:*
			         </Source>
			         <Sink>
			         </Sink>
		         </u:GetProtocolInfoResponse>
	         </s:Body>
         </s:Envelope>".to_string()
    }

    /// Generates the Device Description Response using the Server Configuration
    fn get_device_description(&self) -> String {
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>
            <root xmlns:dlna=\"urn:schemas-dlna-org:device-1-0\" xmlns=\"urn:schemas-upnp-org:device-1-0\">
                <specVersion>
                        <major>1</major>
                        <minor>0</minor>
                </specVersion>
                <URLBase>http://{}:{}/</URLBase>
                <device>
	                <dlna:X_DLNADOC xmlns:dlna=\"urn:schemas-dlna-org:device-1-0\">DMS-1.50</dlna:X_DLNADOC>
	                <dlna:X_DLNADOC xmlns:dlna=\"urn:schemas-dlna-org:device-1-0\">M-DMS-1.50</dlna:X_DLNADOC>
                    <deviceType>urn:schemas-upnp-org:device:MediaServer:1</deviceType>
                    <friendlyName>{}</friendlyName>
                       <manufacturer>Jörn Roddelkopf</manufacturer>
                        <manufacturerURL>https://github.com/zeroexploit/</manufacturerURL>
                        <modelDescription>Simpler Linux Media Server</modelDescription>
                        <modelName>SLMServer</modelName>
                        <modelNumber>01</modelNumber>
                        <modelURL>https://github.com/zeroexploit/slms/</modelURL>
                        <serialNumber>13371337</serialNumber>
                        <UPC>SLMS1337</UPC>
                        <UDN>uuid:{}</UDN>
                        <iconList>
                                <icon>
                                        <mimetype>image/png</mimetype>
                                        <width>120</width>
                                        <height>120</height>
                                        <depth>24</depth>
                                        <url>/files/images/icon.png</url>
                                </icon>
                        </iconList>
                        <presentationURL>http://{}:{}/console/console.html</presentationURL>
                        <serviceList>
                                <service>
                                        <serviceType>urn:schemas-upnp-org:service:ContentDirectory:1</serviceType>
                                        <serviceId>urn:upnp-org:serviceId:ContentDirectory</serviceId>
                                        <SCPDURL>/connection/content_directory.xml</SCPDURL>
                                        <controlURL>/content/content_directory</controlURL>
                                        <eventSubURL>/content/content_directory</eventSubURL>
                              </service>
                                <service>
                                        <serviceType>urn:schemas-upnp-org:service:ConnectionManager:1</serviceType>
                                        <serviceId>urn:upnp-org:serviceId:ConnectionManager</serviceId>
                                        <SCPDURL>/connection/connection_manager.xml</SCPDURL>
                                        <controlURL>/connection/connection_manager</controlURL>
                                        <eventSubURL>/connection/connection_manager</eventSubURL>
                                </service>
                        </serviceList>
                </device>
            </root>",
            self.server_cfg.server_ip,
            self.server_cfg.server_port,
            self.server_cfg.server_name,
            self.server_cfg.server_uuid,
            self.server_cfg.server_ip,
            self.server_cfg.server_port
        )
    }

    /// Get the Connection Manager Service Description
    fn get_connection_manager_description(&self) -> String {
        "<?xml version=\"1.0\"?>
		 <scpd xmlns=\"urn:schemas-upnp-org:service-1-0\">
			 <specVersion>
				 <major>1</major>
				 <minor>0</minor>
			 </specVersion>
			 <actionList>
				 <action>
					 <name>GetProtocolInfo</name>
					 <argumentList>
						 <argument>
							 <name>Source</name>
							 <direction>out</direction>
							 <relatedStateVariable>SourceProtocolInfo</relatedStateVariable>
						 </argument>
						 <argument>
							 <name>Sink</name>
							 <direction>out</direction>
							 <relatedStateVariable>SinkProtocolInfo</relatedStateVariable>
						 </argument>
					 </argumentList>
				 </action>
				 <action>
					 <name>PrepareForConnection</name>
					 <argumentList>
						 <argument>
							 <name>RemoteProtocolInfo</name>
							 <direction>in</direction>
							 <relatedStateVariable>A_ARG_TYPE_ProtocolInfo</relatedStateVariable>
						 </argument>
						 <argument>
							 <name>PeerConnectionManager</name>
							 <direction>in</direction>
							 <relatedStateVariable>A_ARG_TYPE_ConnectionManager</relatedStateVariable>
						 </argument>
						 <argument>
							 <name>PeerConnectionID</name>
							 <direction>in</direction>
							 <relatedStateVariable>A_ARG_TYPE_ConnectionID</relatedStateVariable>
						 </argument>
						 <argument>
							 <name>Direction</name>
							 <direction>in</direction>
							 <relatedStateVariable>A_ARG_TYPE_Direction</relatedStateVariable>
						 </argument>
						 <argument>
							 <name>ConnectionID</name>
							 <direction>out</direction>
							 <relatedStateVariable>A_ARG_TYPE_ConnectionID</relatedStateVariable>
						 </argument>
						 <argument>
							 <name>AVTransportID</name>
							 <direction>out</direction>
						 <relatedStateVariable>A_ARG_TYPE_AVTransportID</relatedStateVariable>
						 </argument>
						 <argument>
							 <name>RcsID</name>
							 <direction>out</direction>
							 <relatedStateVariable>A_ARG_TYPE_RcsID</relatedStateVariable>
						 </argument>
					 </argumentList>
				 </action>
				 <action>
					 <name>ConnectionComplete</name>
					 <argumentList>
						 <argument>
							 <name>ConnectionID</name>
							 <direction>in</direction>
							 <relatedStateVariable>A_ARG_TYPE_ConnectionID</relatedStateVariable>
						 </argument>
					 </argumentList>
				 </action>
				 <action>
					 <name>GetCurrentConnectionIDs</name>
						 <argumentList>
							 <argument>
								 <name>ConnectionIDs</name>
								 <direction>out</direction>
								 <relatedStateVariable>CurrentConnectionIDs</relatedStateVariable>
							 </argument>
						 </argumentList>
				 </action>
				 <action>
					 <name>GetCurrentConnectionInfo</name>
				 <argumentList>
					 <argument>
						 <name>ConnectionID</name>
						 <direction>in</direction>
						 <relatedStateVariable>A_ARG_TYPE_ConnectionID</relatedStateVariable>
					 </argument>
					 <argument>
						 <name>RcsID</name>
						 <direction>out</direction>
						 <relatedStateVariable>A_ARG_TYPE_RcsID</relatedStateVariable>
					 </argument>
					 <argument>
						 <name>AVTransportID</name>
						 <direction>out</direction>
						 <relatedStateVariable>A_ARG_TYPE_AVTransportID</relatedStateVariable>
					 </argument>
					 <argument>
						 <name>ProtocolInfo</name>
						 <direction>out</direction>
						 <relatedStateVariable>A_ARG_TYPE_ProtocolInfo</relatedStateVariable>
					 </argument>
					 <argument>
						 <name>PeerConnectionManager</name>
						 <direction>out</direction>
						 <relatedStateVariable>A_ARG_TYPE_ConnectionManager</relatedStateVariable>
					 </argument>
					 <argument> 
						 <name>PeerConnectionID</name>
						 <direction>out</direction>
						 <relatedStateVariable>A_ARG_TYPE_ConnectionID</relatedStateVariable>
					 </argument>
					 <argument>
						 <name>Direction</name>
						 <direction>out</direction>
						 <relatedStateVariable>A_ARG_TYPE_Direction</relatedStateVariable>
					 </argument>
					 <argument>
						 <name>Status</name>
						 <direction>out</direction>
						 <relatedStateVariable>A_ARG_TYPE_ConnectionStatus</relatedStateVariable>
					 </argument>
				 </argumentList>
			 </action>
		</actionList>
		<serviceStateTable>
			<stateVariable sendEvents=\"yes\">
				<name>SourceProtocolInfo</name>
				<dataType>string</dataType>
			</stateVariable>
			<stateVariable sendEvents=\"yes\">
				<name>SinkProtocolInfo</name>
				<dataType>string</dataType>
			</stateVariable>
			<stateVariable sendEvents=\"yes\">
				 <name>CurrentConnectionIDs</name>
				 <dataType>string</dataType>
			</stateVariable>
			<stateVariable sendEvents=\"no\">
				<name>A_ARG_TYPE_ConnectionStatus</name>
				<dataType>string</dataType>
				<allowedValueList>
					 <allowedValue>OK</allowedValue>
					 <allowedValue>ContentFormatMismatch</allowedValue>
					 <allowedValue>InsufficientBandwidth</allowedValue>
					 <allowedValue>UnreliableChannel</allowedValue>
					 <allowedValue>Unknown</allowedValue>
				 </allowedValueList>
			 </stateVariable>
			 <stateVariable sendEvents=\"no\">
				 <name>A_ARG_TYPE_ConnectionManager</name>
				 <dataType>string</dataType>
			 </stateVariable>
			 <stateVariable sendEvents=\"no\">
				 <name>A_ARG_TYPE_Direction</name>
				 <dataType>string</dataType>
				 <allowedValueList>
					 <allowedValue>Input</allowedValue>
					 <allowedValue>Output</allowedValue>
				 </allowedValueList>
			 </stateVariable>
			 <stateVariable sendEvents=\"no\">
				 <name>A_ARG_TYPE_ProtocolInfo</name>
				 <dataType>string</dataType>
			 </stateVariable>
			 <stateVariable sendEvents=\"no\">
				 <name>A_ARG_TYPE_ConnectionID</name>
				 <dataType>i4</dataType> 
			 </stateVariable>
			 <stateVariable sendEvents=\"no\">
				 <name>A_ARG_TYPE_AVTransportID</name>
				 <dataType>i4</dataType>
			 </stateVariable>
			 <stateVariable sendEvents=\"no\">
				 <name>A_ARG_TYPE_RcsID</name>
				 <dataType>i4</dataType>
			 </stateVariable>
		</serviceStateTable>
	</scpd>"
            .to_string()
    }

    /// Get the Content Directory Service Description
    fn get_content_directory_description(&self) -> String {
        "<?xml version=\"1.0\"?>
		<scpd xmlns=\"urn:schemas-upnp-org:service-1-0\">
			 <specVersion>
			 <major>1</major>
			 <minor>0</minor>
			 </specVersion>
			 <actionList>
			 <action>
			 <name>GetSearchCapabilities</name>
			 <argumentList>
			 <argument>
			 <name>SearchCaps</name>
			<direction>out</direction>
			 <relatedStateVariable>SearchCapabilities</relatedStateVariable>
			 </argument>
			 </argumentList>
			 </action>
			 <action>
			 <name>GetSortCapabilities</name>
			 <argumentList>
			 <argument>
			 <name>SortCaps</name>
			 <direction>out</direction>
			 <relatedStateVariable>SortCapabilities</relatedStateVariable>
			 </argument>
			</argumentList>
			 </action>
			 <action>
			 <name>GetSystemUpdateID</name>
			 <argumentList>
			 <argument>
			 <name>Id</name>
			 <direction>out</direction>
			 <relatedStateVariable>SystemUpdateID</relatedStateVariable>
			 </argument>
			 </argumentList>
			 </action>
			 <action>
			 <name>Browse</name>
			 <argumentList>
			 <argument>
			 <name>ObjectID</name>
			 <direction>in</direction>
			 <relatedStateVariable>A_ARG_TYPE_ObjectID</relatedStateVariable>
			 </argument>
			 <argument>
			 <name>BrowseFlag</name>
			 <direction>in</direction>
			 <relatedStateVariable>A_ARG_TYPE_BrowseFlag</relatedStateVariable>
			</argument>
			 <argument>
			 <name>Filter</name>
			 <direction>in</direction> 
			 <relatedStateVariable>A_ARG_TYPE_Filter</relatedStateVariable>
			 </argument>
			 <argument>
			 <name>StartingIndex</name>
			 <direction>in</direction>
			<relatedStateVariable>A_ARG_TYPE_Index</relatedStateVariable>
			 </argument>
			 <argument>
			 <name>RequestedCount</name>
			 <direction>in</direction>
			 <relatedStateVariable>A_ARG_TYPE_Count</relatedStateVariable>
			 </argument>
			 <argument>
			 <name>SortCriteria</name>
			 <direction>in</direction>
			 <relatedStateVariable>A_ARG_TYPE_SortCriteria</relatedStateVariable>
			 </argument>
			 <argument>
			 <name>Result</name>
			 <direction>out</direction>
			 <relatedStateVariable>A_ARG_TYPE_Result</relatedStateVariable>
			 </argument>
			 <argument>
			 <name>NumberReturned</name>
			 <direction>out</direction>
			 <relatedStateVariable>A_ARG_TYPE_Count</relatedStateVariable>
			 </argument>
			 <argument>
			 <name>TotalMatches</name>
			 <direction>out</direction>
			 <relatedStateVariable>A_ARG_TYPE_Count</relatedStateVariable>
			</argument>
			 <argument>
			 <name>UpdateID</name>
			 <direction>out</direction>
			 <relatedStateVariable>A_ARG_TYPE_UpdateID</relatedStateVariable>
			 </argument>
			 </argumentList>
			 </action>
			 <action>
			 <name>Search</name>
			 <argumentList>
			 <argument>
			 <name>ContainerID</name>
			 <direction>in</direction>
			 <relatedStateVariable>A_ARG_TYPE_ObjectID</relatedStateVariable>
			 </argument>
			 <argument>
			 <name>SearchCriteria</name>
			 <direction>in</direction>
			 <relatedStateVariable>A_ARG_TYPE_SearchCriteria</relatedStateVariable>
			 </argument>
			 <argument>
			 <name>Filter</name>
			 <direction>in</direction>
			 <relatedStateVariable>A_ARG_TYPE_Filter</relatedStateVariable> 
			 </argument>
			 <argument>
			 <name>StartingIndex</name>
			 <direction>in</direction>
			 <relatedStateVariable>A_ARG_TYPE_Index</relatedStateVariable>
			</argument>
			 <argument>
			 <name>RequestedCount</name>
			 <direction>in</direction>
			 <relatedStateVariable>A_ARG_TYPE_Count</relatedStateVariable>
			 </argument>
			 <argument>
			 <name>SortCriteria</name>
			 <direction>in</direction>
			 <relatedStateVariable>A_ARG_TYPE_SortCriteria</relatedStateVariable>
			 </argument>
			 <argument>
			 <name>Result</name>
			 <direction>out</direction>
			 <relatedStateVariable>A_ARG_TYPE_Result</relatedStateVariable>
			 </argument>
			 <argument>
			 <name>NumberReturned</name>
			 <direction>out</direction>
			 <relatedStateVariable>A_ARG_TYPE_Count</relatedStateVariable>
			 </argument>
			 <argument>
			 <name>TotalMatches</name>
			 <direction>out</direction>
			 <relatedStateVariable>A_ARG_TYPE_Count</relatedStateVariable>
			 </argument>
			 <argument>
			 <name>UpdateID</name>
			 <direction>out</direction>
			 <relatedStateVariable>A_ARG_TYPE_UpdateID</relatedStateVariable>
			 </argument>
			 </argumentList>
			 </action>
			 <action>
			 <name>CreateObject</name>
			 <argumentList>
			 <argument>
			 <name>ContainerID</name>
			 <direction>in</direction>
			 <relatedStateVariable>A_ARG_TYPE_ObjectID</relatedStateVariable>
			 </argument>
			 <argument>
			 <name>Elements</name>
			 <direction>in</direction>
			 <relatedStateVariable>A_ARG_TYPE_Result</relatedStateVariable>
			 </argument>
			 <argument>
			 <name>ObjectID</name>
			 <direction>out</direction>
			 <relatedStateVariable>A_ARG_TYPE_ObjectID</relatedStateVariable>
			 </argument> 
			 <argument>
			 <name>Result</name>
			 <direction>out</direction>
			 <relatedStateVariable>A_ARG_TYPE_Result</relatedStateVariable>
			 </argument>
			</argumentList>
			 </action>
			 <action>
			 <name>DestroyObject</name>
			 <argumentList>
			 <argument>
			 <name>ObjectID</name>
			 <direction>in</direction>
			 <relatedStateVariable>A_ARG_TYPE_ObjectID</relatedStateVariable>
			 </argument>
			 </argumentList>
			 </action>
			 <action>
			 <name>UpdateObject</name>
			 <argumentList>
			 <argument>
			 <name>ObjectID</name>
			 <direction>in</direction>
			 <relatedStateVariable>A_ARG_TYPE_ObjectID</relatedStateVariable>
			 </argument>
			 <argument>
			 <name>CurrentTagValue</name>
			 <direction>in</direction>
			 <relatedStateVariable>A_ARG_TYPE_TagValueList</relatedStateVariable>
			 </argument>
			 <argument>
			 <name>NewTagValue</name>
			 <direction>in</direction>
			 <relatedStateVariable>A_ARG_TYPE_TagValueList</relatedStateVariable>
			 </argument>
			 </argumentList>
			 </action>
			 <action>
			 <name>ImportResource</name>
			 <argumentList>
			 <argument>
			 <name>SourceURI</name>
			 <direction>in</direction>
			 <relatedStateVariable>A_ARG_TYPE_URI</relatedStateVariable>
			 </argument>
			 <argument>
			 <name>DestinationURI</name>
			 <direction>in</direction>
			 <relatedStateVariable>A_ARG_TYPE_URI</relatedStateVariable>
			 </argument>
			 <argument>
			 <name>TransferID</name>
			 <direction>out</direction>
			 <relatedStateVariable>A_ARG_TYPE_TransferID</relatedStateVariable>
			 </argument>
			 </argumentList> 
			 </action>
			 <action>
			 <name>ExportResource</name>
			 <argumentList>
			 <argument>
			 <name>SourceURI</name>
			 <direction>in</direction>
			 <relatedStateVariable>A_ARG_TYPE_URI</relatedStateVariable>
			 </argument>
			 <argument>
			 <name>DestinationURI</name>
			 <direction>in</direction>
			 <relatedStateVariable>A_ARG_TYPE_URI</relatedStateVariable>
			 </argument>
			 <argument>
			 <name>TransferID</name>
			 <direction>out</direction>
			 <relatedStateVariable>A_ARG_TYPE_TransferID</relatedStateVariable>
			</argument>
			 </argumentList>
			 </action>
			 <action>
			 <name>StopTransferResource</name>
			 <argumentList>
			 <argument>
			 <name>TransferID</name>
			 <direction>in</direction>
			 <relatedStateVariable>A_ARG_TYPE_TransferID</relatedStateVariable>
			 </argument>
			 </argumentList>
			 </action>
			 <action>
			 <name>GetTransferProgress</name>
			 <argumentList>
			 <argument>
			 <name>TransferID</name>
			 <direction>in</direction>
			 <relatedStateVariable>A_ARG_TYPE_TransferID</relatedStateVariable>
			 </argument>
			 <argument>
			 <name>TransferStatus</name>
			 <direction>out</direction>
			 <relatedStateVariable>A_ARG_TYPE_TransferStatus</relatedStateVariable>
			 </argument>
			 <argument>
			 <name>TransferLength</name>
			 <direction>out</direction>
			 <relatedStateVariable>A_ARG_TYPE_TransferLength</relatedStateVariable>
			 </argument>
			 <argument>
			 <name>TransferTotal</name>
			 <direction>out</direction>
			 <relatedStateVariable>A_ARG_TYPE_TransferTotal</relatedStateVariable>
			 </argument>
			 </argumentList>
			 </action> 
			 <action>
			 <name>DeleteResource</name>
			 <argumentList>
			 <argument>
			 <name>ResourceURI</name>
			 <direction>in</direction>
			 <relatedStateVariable>A_ARG_TYPE_URI</relatedStateVariable>
			 </argument>
			 </argumentList>
			 </action>
			 <action>
			 <name>CreateReference</name>
			 <argumentList>
			 <argument>
			 <name>ContainerID</name>
			 <direction>in</direction>
			 <relatedStateVariable>A_ARG_TYPE_ObjectID</relatedStateVariable>
			 </argument>
			 <argument>
			 <name>ObjectID</name>
			 <direction>in</direction>
			 <relatedStateVariable>A_ARG_TYPE_ObjectID</relatedStateVariable>
			 </argument>
			 <argument>
			 <name>NewID</name>
			 <direction>out</direction>
			 <relatedStateVariable>A_ARG_TYPE_ObjectID</relatedStateVariable>
			 </argument>
			 </argumentList>
			 </action> 
			 </actionList>
			 <serviceStateTable>
			 <stateVariable sendEvents=\"yes\">
			 <name>TransferIDs</name>
			 <dataType>string</dataType>
			 </stateVariable>
			 <stateVariable sendEvents=\"no\">
			 <name>A_ARG_TYPE_ObjectID</name>
			 <dataType>string</dataType>
			 </stateVariable>
			 <stateVariable sendEvents=\"no\">
			 <name>A_ARG_TYPE_Result</name>
			 <dataType>string</dataType>
			 </stateVariable>
			 <stateVariable sendEvents=\"no\">
			 <name>A_ARG_TYPE_SearchCriteria</name>
			 <dataType>string</dataType>
			 </stateVariable>
			 <stateVariable sendEvents=\"no\">
			 <name>A_ARG_TYPE_BrowseFlag</name>
			 <dataType>string</dataType>
			<allowedValueList>
			 <allowedValue>BrowseMetadata</allowedValue>
			 <allowedValue>BrowseDirectChildren</allowedValue>
			 </allowedValueList> 
			 </stateVariable>
			 <stateVariable sendEvents=\"no\">
			 <name>A_ARG_TYPE_Filter</name>
			 <dataType>string</dataType>
			 </stateVariable>
			 <stateVariable sendEvents=\"no\">
			 <name>A_ARG_TYPE_SortCriteria</name>
			 <dataType>string</dataType>
			 </stateVariable>
			 <stateVariable sendEvents=\"no\">
			 <name>A_ARG_TYPE_Index</name>
			 <dataType>ui4</dataType>
			 </stateVariable>
			 <stateVariable sendEvents=\"no\">
			 <name>A_ARG_TYPE_Count</name>
			 <dataType>ui4</dataType>
			 </stateVariable>
			 <stateVariable sendEvents=\"no\">
			 <name>A_ARG_TYPE_UpdateID</name>
			 <dataType>ui4</dataType>
			 </stateVariable>
			 <stateVariable sendEvents=\"no\">
			 <name>A_ARG_TYPE_TransferID</name>
			 <dataType>ui4</dataType>
			 </stateVariable>
			 <stateVariable sendEvents=\"no\">
			 <name>A_ARG_TYPE_TransferStatus</name>
			 <dataType>string</dataType>
			 <allowedValueList>
			 <allowedValue>COMPLETED</allowedValue>
			 <allowedValue>ERROR</allowedValue>
			 <allowedValue>IN_PROGRESS</allowedValue>
			 <allowedValue>STOPPED</allowedValue>
			 </allowedValueList>
			 </stateVariable>
			 <stateVariable sendEvents=\"no\">
			 <name>A_ARG_TYPE_TransferLength</name>
			 <dataType>string</dataType>
			 </stateVariable>
			 <stateVariable sendEvents=\"no\">
			 <name>A_ARG_TYPE_TransferTotal</name>
			 <dataType>string</dataType>
			 </stateVariable>
			 <stateVariable sendEvents=\"no\">
			 <name>A_ARG_TYPE_TagValueList</name>
			 <dataType>string</dataType>
			 </stateVariable>
			 <stateVariable sendEvents=\"no\">
			 <name>A_ARG_TYPE_URI</name>
			 <dataType>uri</dataType>
			 </stateVariable>
			 <stateVariable sendEvents=\"no\">
			 <name>SearchCapabilities</name>
			 <dataType>string</dataType>
			 </stateVariable>
			 <stateVariable sendEvents=\"no\">
			 <name>SortCapabilities</name>
			 <dataType>string</dataType>
			 </stateVariable>
			 <stateVariable sendEvents=\"yes\">
			 <name>SystemUpdateID</name>
			<dataType>ui4</dataType>
			 </stateVariable>
			 <stateVariable sendEvents=\"yes\">
			 <name>ContainerUpdateIDs</name>
			 <dataType>string</dataType>
			 </stateVariable> 
			 </serviceStateTable>
		</scpd>"
            .to_string()
    }
}
