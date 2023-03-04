# SkyTree

SkyTree is a web application that exposes both a web interface and a REST API for managing an Ansible database inventory. 

The default Ansible inventory is a flat text file, very simple, that just lists servers and variables. It lacks a bit of structure, needed for more advanced use cases. 
Fortunately, the inventory system is pluggable, so it is rather easy to replace it with something a bit more structured.

The database inventory system exposes these concepts:
 - Hierarchical server groups, with group variables
 - Hierarchical services, with service variables
 - Servers, with server variables
 - Service instances, an application of a service into a server, namely defining the network endpoint of the service instance

All of these are exposed to roles running under Ansible so they can configure service instances on servers with the correct context.

This repository does not contain the Ansible database inventory plugin. It contains the management web service for the database inventory.
 
