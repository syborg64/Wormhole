### **BETA TEST PLAN – Wormhole**

## **1. Core Functionalities for Beta Version**
[List and describe the core functionalities that must be available for beta testing. Explain any changes made since the original Tech3 Action Plan.]

| **Feature Name**              | **Description**                                                                                       | **Priority (High/Medium/Low)**                | **Changes Since Tech3**                                               |
| ----------------------------- | ----------------------------------------------------------------------------------------------------- | --------------------------------------------- | --------------------------------------------------------------------- |
| Basic Local Interactions      | Read, Write, Move, Rename, Delete files and Folders. All common user interactions must be implemented | **<span style="color: red">High</span>**      | Improved stability                                                    |
| Docker Image                  | Necessary to simplify the user interaction                                                            | **<span style="color: orange">Medium</span>** | [Modifications or additions]                                          |
| Configuration Files           | Wormhole is configurable by configuration file.                                                       | **<span style="color: orange">Medium</span>** | [Modifications or additions]                                          |
| Stability (Almost)            | Most of the Fuser Calls must be implemented to allow the support in all environments                  | **<span style="color: red">High</span>**      | [Modifications or additions]                                          |
| Linux Support                 | Complete support  for all majors Linux systems (Debian, Arch, etc.).                                  | **<span style="color: red">High</span>**      | [Modifications or additions]                                          |
| Mac Support (lack testing)    | Partial support of the mac system.                                                                    | **<span style="color: yellow">Low</span>**    | [Modifications or additions]                                          |
| Windows Support               | 70% of support of the windows system, with incomplete fonctionnality                                  | **<span style="color: red">High</span>**      | Start the implementation                                              |
| Complete User Documentation   | 50% of the user documentation is implemented.                                                         | **<span style="color: red">High</span>**      | [Modifications or additions]                                          |
| Redudancy                     | Basic redundancy with a minimum of 3 pods.                                                            | **<span style="color: red">High</span>**      | Update redudancy (2 pods to a theoretically unlimited number of pods) |
| Compliant Error Handling      | Software interacting with Wormhole may respond accordingly                                            | **<span style="color: orange">Medium</span>** | [Modifications or additions]                                          |
| Cache Handling                | Optimized cache management for improved performance.                                                  | **<span style="color: orange">Medium</span>** | [Modifications or additions]                                          |
| CLI Interface                 | Optimized cache management for improved performance.                                                  | **<span style="color: orange">Medium</span>** | [Modifications or additions]                                          |
| Error Resilience              | not-catastrophic errors can be recovered from                                                         | **<span style="color: red">Hight</span>**     | [Modifications or additions]                                          |
| Fancy optimization techniques | Traffic optimization depending on newtork speed or storage available                                  | **<span style="color: orange">Medium</span>** | **<span style="color: red">Aborted feature</span>**                   |
| Pod specialisation            | Permission, Storage Limit, Redudancy                                                                  | **<span style="color: orange">Medium</span>** | **<span style="color: red">Aborted feature</span>**                   |

---

## **2. Beta Testing Scenarios**
### **2.1 User Roles**
[Define the different user roles that will be involved in testing, e.g., Admin, Regular User, Guest, External Partner.]

| **Role Name**                | **Description**                                                                                                                                                                                                                                                    |
| ---------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| **Windows user**             | Allows us to have some insight on the windows experience, bugs unique to windows                                                                                                                                                                                   |
| **Home server user**         | A home server user with a large archive of files, so we can have a real insight on how our system handles realistic work loads. We could do speed tests and compare to other filesystems to verify efficiency.                                                     |
| **Professionnal Developper** | API Feedback from experienced user, Point of view from a professionnal. Working with a professionnal will allow us to have feedback with the interest of company in mind. It will also let us work with someone with professional experience so a wider skill set. |
| **Open source Contributor**  | Feedback from a developper, for see if the project documentation, test suite, are easy to use and understand by a developper.                                                                                                                                      |
| **Mac user**                 | Allows us to have some insight on the mac experience, bugs unique to mac. A mac user may have high standards for interface, and would give important feedback.                                                                                                     |

### **2.2 Test Scenarios**
For each core functionality, provide detailed test scenarios.

#### **Scenario 1: First Installation**
- **Role Involved:** For every role
- **Objective:** Verify that the process for a new user to start using wormhole is simple and clear.
- **Prerequisites:** None
- **Test Steps:**
  1. Get access to the wormhole documentation and installation page
  2. Create a network
  3. Create a second pod on the same machine, connected to the first
  4. (For advanced users) Create a third instance on the same local area network, and connect it to the others
  5. Test that all instances are properly connected by adding a blank file to the network
- **Expected Outcome:** The user doesn't need to consult any external resources for installation, and has a functional network

#### **Scenario 2: Basic Local Interactions**
- **Role Involved:** For every role
- **Objective:** Interact with the filesystem (read, write, move, rename, delete, ...) a file or folder.
- **Prerequisites:** Installing Wormhole With a single instance
- **Test Steps:**
  1. Create a file
  2. Create a folder
  3. Write to a file
  4. Append to a file
  5. Read a file
  6. Create files into a folder
  7. Create folders into a folder
  8. Move a file into an other folder
  9. Move a folder into an other folder
  10. Move a file from an external filesystem
  11. Move a file to an external filesystem
  12. Move a folder from an external filesystem
  13. Move a folder  a to an external filesystem
  14. Rename a file
  15. Rename a folder
  16. Delete a file
  17. Delete a folder
  18. Delete a folder recursively
  19. List the content of a folder
  20. Get metadata for a file or folder
  21. Change permissions of file, retry previous relevant steps
  22. Change permissions of a folder, retry previous relevant steps
- **Expected Outcome:** No errors surprise the user. Actions that would be illegal on any filesystem return the correct error

#### **Scenario _: Complex interactions**
... Run an app inside a Wormhole instance, check the result on an other connected instance

#### **Scenario 3: Docker Image**
- **Role Involved:** Professionnal developper, home server user
- **Objective:** Testing wormhole with a docker image
- **Prerequisites:** Installing Wormhole and Docker
- **Test Steps:**
  1. Pull the Wormhole repo and try to build and run the provided simple Wormhole Docker image.
  2. Launch the Docker container with the appropriate command.
  3. Check that Wormhole mount correctly by consulting the logs or the container status.
  4. Performs tests similar to the Scenario 2 (Basic local interactions)
  5. Spawn a second docker image on another machine, and check that the connection is correctly work.
open-source app that uses the provided Wormhole filesystem.  6. Check for errors on the utilisation of this third party app.
- **Expected Outcome:** The Docker image runs Wormhole correctly, and all basic file system operations run smoothly. Any errors are easily viewed through the docker container. Third party apps can function using the container.

#### **Scenario 4: Configuration Files**
- **Role Involved:** For every role
- **Objective:** Create a configuration file based on the documentation and use it.
- **Prerequisites:** Installing Wormhole
- **Test Steps:**
  1. Use the Wormhole CLI to generate a default configuration file.
  2. Modify one setting (e.g. mount point or redundancy parameters).
  3. Start Wormhole with the modified configuration and check that the changes are in place.
  4. Modify another setting (e.g. cache settings).
  5. Reload configuration using the CLI without restarting Wormhole.
  6. Check that the new configuration has been applied
- **Expected Outcome:** Documentation is clear and easy to use. The configuration is really corectly used before and after the reload.

#### **Scenario 5: Stability**
- **Role Involved:** For every role
- **Objective:** Test Wormhole's stability under various operations without crashes or critical bugs.
- **Prerequisites:** Installing Wormhole and configuring a basic instance
- **Test Steps:**
  1. Perform a rapid series of operations: create several files, write, read, delete, rename.
  2. Attempt operations likely to generate errors (e.g. deleting a non-existent file).
  3. Run Wormhole for 24 hours with periodic operations.
  4. Upload and download large files (e.g. 5GB).
  5. Monitor logs for errors or warnings.
  6. Interrupt Wormhole (e.g. sudden stop) and restart to verify recovery.
  7. Interrupt Wormhole instance and check if the owned files are still available
- **Expected Outcome:** Wormhole manages all operations without crashing, logs errors appropriately and maintains data integrity.

#### **Scenario 6: Linux Support**
- **Role Involved:** Home server user
- **Objective:** Testing if Wormhole works on Linux systems (Debian, Ubuntu, etc.)
- **Prerequisites:** Installing Wormhole
- **Test Steps:**
  1. Install Wormhole on a major Linux distribution.
  2. Execute common operations on a filesystem (creating, reading and editing files).
  3. Use and test advanced features with configuration files like redundancy.
  4. Use this system accross many of your servers, with external apps that could benefit from large distributed storage (ex. storage of pictures like Immich), and see if they are able to operate normally.
- **Expected Outcome:** Wormhole installs and runs correctly on several Linux distributions, and can be used by other programs that require file storage.

#### **Scenario 7: Mac Support**
- **Role Involved:** Mac user
- **Objective:** Testing if Wormhole works on Mac systems.
- **Prerequisites:** Installing Wormhole
- **Test Steps:**
  1. Install Wormhole on a macOS system.
  2. Perform basic operations: create, read, write and delete files and folders.
  3. Check macOS-specific issues (e.g. permissions, Finder integration).
  4. Test the user interface to ensure it is intuitive for Mac users.
- **Expected Outcome:** Wormhole runs on macOS with the correct basic operations, although limitations may exist due to not currently being officially supported.

#### **Scenario 8: Window Support**
- **Role Involved:** Windows user
- **Objective:** Testing if Wormhole works on Windows systems.
- **Prerequisites:** Installing Wormhole
- **Test Steps:**
  1. Install Wormhole on a Windows system (e.g. Windows 10 or 11).
  2. Perform basic operations: create, read, write, delete files and folders.
  3. Test Windows-specific integrations.
  4. Identify unimplemented or buggy features (linked to 70% status).
- **Expected Outcome:** Wormhole installs and runs on Windows, with most basic features operational, but possible limitations or bugs to be documented.

#### **Scenario 9: Complete User Documentation**
- **Role Involved:** For every role
- **Objective:** Check if the documentation is clear and easy to use for a new user.
- **Prerequisites:** No required setup
- **Test Steps:**
  1. Read the documentation to install Wormhole
  2. Read the documentation to use Wormhole
  3. Read the documentation to configure Wormhole
  4. Identify incomplete or unclear sections.
  5. Suggest improvements or additions to the content.
- **Expected Outcome:** If the documentation is clear and easy to use, the user should be able to install, use and configure Wormhole.

#### **Scenario 10: Redudancy**
- **Role Involved:** For every role
- **Objective:** Testing the redundancy of the system.
- **Prerequisites:** Installing Wormhole on 3 different machines
- **Test Steps:**
  1. Create a Wormhole instance on each machine in the same Wormhole network.
  2. Create a different file on each machine.
  3. Check that the files are available on all machines.
  4. Shutdown machine *A*.
  5. Check that the files remain available on the remaining machines (machine *B* must retrieve the file from *A*).
  6. Modify machine *A*'s file on machine *B*.
  7. Restart machine *A*.
  8. Check that the modified file is updated on machine *A*.
- **Expected Outcome:** The system maintains file availability and consistency, even in the event of a node failure, and changes are propagated correctly.

#### **Scenario 10: Clean Error**
- **Role Involved:** For every role
- **Objective:** Testing the error handling of the system are complete and understandable by the user.
- **Prerequisites:** Create a Wormhole network
- **Test Steps:**
  1. Attempt an operation that should fail (e.g. write to a read-only file or access a non-existent file).
  2. Observe the error message or feedback provided by Wormhole.
  3. Check that the message is clear and helps to understand the problem and its resolution.
  4. Test error handling in different contexts.
  5. Check that errors are logged for debugging purposes.
- **Expected Outcome:** Wormhole provides clear and useful error messages that facilitate troubleshooting without confusing the user.

#### **Senario 10: CLI Interface**
- **Role Involved:** For every role
- **Objective:**
- **Prerequisites:**
- **Test Steps:**
  1. Step 1
- **Expected Outcome:**

#### **Senario 10: Error Resilience**
- **Role Involved:** For every role
- **Objective:**
- **Prerequisites:**
- **Test Steps:**
  1. Step 1
- **Expected Outcome:**
---

## **3. Success Criteria**
[Define the metrics and conditions that determine if the beta version is successful.]

The following criteria will be used to determine the success of the beta version.

| **Criterion** | **Description**                                                         | **Threshold for Success**              |
| ------------- | ----------------------------------------------------------------------- | -------------------------------------- |
| Stability     | No major crashes or critical bugs                                       | No crash reported                      |
| Usability     | Users cny cloud hosted d                                                | 80% positive feedback from testers     |
| Performance   | 95% of individual files operation are completed in less than 10 seconds | 95% of files analyzed within 2 seconds |
| Accuracy      | 90% des cas de test r                                                   | 90% accuracy in test cases             |

Compliantce

---

## **4. Known Issues & Limitations**
[List any known bugs, incomplete features, or limitations that testers should be aware of.]

| **Issue**      | **Description**                                 | **Impact**                                    | **Planned Fix? (Yes/No)** |
| -------------- | ----------------------------------------------- | --------------------------------------------- | ------------------------- |
| Window support | Incomplete support of the windows system        | **<span style="color: red">High</span>**      | Yes                       |
| Documentation  | Incomplete documentation for user or developper | **<span style="color: orange">Medium</span>** | Yes                       |

---

## **5. Conclusion**
This **Beta Test Plan** for Wormhole describes the essential steps for testing core functionality across different user roles and scenarios. By involving a variety of testers (Mac users, Windows users, home servers, professional engineers and developers), we aim to gather comprehensive feedback on usability, stability and performance. The success criteria defined will enable us to assess whether Wormhole is ready for a wider release. Resolving known issues and limitations during the beta phase will be crucial to delivering a robust, user-friendly product. We hope that the insights gained from these tests will guide final adjustments and ensure that Wormhole meets the high expectations of our users.