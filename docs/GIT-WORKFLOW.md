# Git - Workflow 
 
The following Lines of Text will describe how this Git-Project is organized and what kind of Workflow is used in Order to develop the Simple Linux Media Server. But, everything written here, might be already outdated as you are reading this. The SLMS Project is just a nice little Piece of Software i am using to experiment with Git and different Ways of Software Development. Therefore, the following Approaches might be modified, updated or discarded at any time and without notice. 
If you think the Development Process could be improved, feel free to let me know about it. 
 
 
## Overview 
 
The main idea is to use feature branches to develop single parts of the software, merge them into a dev branch once they are done and create a release branch when all features are included. Than on that Branch all stabilization is done. When a release candidate enters a stable state, it will be pulled to the master branch and tagged as current stable version. 
 
 
## Branches 
 
### Feature Branches 
 
Feature Branches will be created from the dev Branch to develop single Parts ("Features") of the Application. All the development work shall happen only on the Feature Branch and for a single feature only. Once a Feature is "done" it has to be included in the dev Branch through a Pull Request and the Feature Branch will be deleted. Feature Branches are always named with a "feature/*" prefix, followed by the module name to develop. 
 
### Dev Branch 
 
The dev Branch will be used to hold the current State of the overall Development Progress as all Features get pulled on that one. No actual development should happen on here. Once a new Feature is required, a new Feature Branch will be created originating from the dev Branch. 
 
### Doc Branch 
 
The Doc Branch is used to create all Documentation related Files and Descriptions. No Source Code should be developed here. In addition to that, Source Code Comments are not meant to be part of this Branch either. When a Release Branch is created, the doc Branch will be pulled too. 
 
### Master Branch 
 
The Master Branch stores the latest stable Version of the Software and contains only Tagged Releases. No Development happens here and this Branch can always be build to the latest stable Version. If any kind of hotfix is required, a new Branch originating from Master will be created and used for that. 
 
### Release Branches 
 
When the dev Branch reaches a releasable state a new branch will be created originating from dev and pulling in doc. Then all stabilization is done on that branch but no other features will be included. Once the Release is stable a Pull-Request is used to merge it to the master and create a new Release. Another Pull Request is used to transmit any stabilization back to the dev Branch. The Release Branch than is deleted. Using this approach it is possible to work on the next release while stabilizing the last one. 
 
### Hotfix Branches 
 
If hotfixes are required, a new branch originating from the master will be created, the hotfix will be applied, pulled in the master and to the dev branch. Once it is done the Hotfix Branch will be deleted.
 
