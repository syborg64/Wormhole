### **BETA TEST PLAN – Wormhole**

## **1. Core Functionalities for Beta Version**
[List and describe the core functionalities that must be available for beta testing. Explain any changes made since the original Tech3 Action Plan.]

| **Feature Name**             | **Description**                                                                                       | **Priority (High/Medium/Low)**                | **Changes Since Tech3**                            |
| ---------------------------- | ----------------------------------------------------------------------------------------------------- | --------------------------------------------- | -------------------------------------------------- |
| Basic shell commands         | Read, Write, Move, Rename, Delete files and Folders. All common user interactions must be implemented | **<span style="color: red">High</span>**      | [Modifications or additions]                       |
| Docker Image                 | Necessary to simplify the user interaction                                                            | **<span style="color: orange">Medium</span>** | [Modifications or additions]                       |
| Configuration files          | Wormhole is confugrable by configuration file.                                                        | **<span style="color: red">High</span>**      | [Modifications or additions]                       |
| Stability (Almost)           | Most of the Fuser Calls must be implemented to allow the best interaction with interfaces             | **<span style="color: red">High</span>**      | [Modifications or additions]                       |
| Linux Support                | Complet support  for all Linux systems (Debian, Ubuntu, etc.).                                        | **<span style="color: red">High</span>**      | [Modifications or additions]                       |
| Mac Support (lack testing)   | Partial support of the mac system.                                                                    | **<span style="color: yellow">Low</span>**    | [Modifications or additions]                       |
| Windows Support              | 70% of support of the windows system, with incomplete fonctionnality                                  | **<span style="color: red">High</span>**      | [Modifications or additions]                       |
| Complete User Documentation  | 50% of the user documentation is implemented.                                                         | **<span style="color: red">High</span>**      | [Modifications or additions]                       |
| Redudancy                    | Basic redundancy with a minimum of 3 pods.                                                            | **<span style="color: red">High</span>**      | [Modifications or additions]                       |
| Clean Error Handling         | Mostly only used by TUI but will help with feedback                                                   | **<span style="color: orange">Medium</span>** | [Modifications or additions]                       |
| Cache handling               | Optimized cache management for improved performance.                                                  | **<span style="color: orange">Medium</span>** | [Modifications or additions]                       |
| Fancy optimization technique | Trafic optimization dependig on internet speed or storage left                                        | **<span style="color: orange">Medium</span>** | **<span style="color: red">Abored feature</span>** |
| Pod specialisation           | Permission, Storage Limite, Redudancy                                                                 | **<span style="color: orange">Medium</span>** | **<span style="color: red">Abored feature</span>** |

---

## **2. Beta Testing Scenarios**
### **2.1 User Roles**
[Define the different user roles that will be involved in testing, e.g., Admin, Regular User, Guest, External Partner.]

| **Role Name**              | **Description**                                                                                                                                                                                                                                                   |
| -------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Mac user**               | Allow us to have some insight on the mac experience, bugs unique to mac. A mac user is also used to easy to use interface so we might have usefull API feedback.                                                                                                  |
| **Windows user**           | Allow us to have some insight on the windows experience, bugs unique to windows                                                                                                                                                                                   |
| **Home server user**       | A home server user with a large catalogue of files, so wa can have a real insight of how our system handle realistic hight work load. We could do speed tests and comporate to other filesystems to verify efficiency.                                            |
| **Professionnal engineer** | API Feedback from experienced user, Point of vue from a professionnal. Working with a professionnal will allow us to have feedback with the interest of company in mind. It will also let us work with someone with professional experience so a wider skill set. |
| **Wormhole Developper**    | Feedback from a developper, for see if the project is easy to use and understand by a developper.                                                                                                                                                                 |

### **2.2 Test Scenarios**
For each core functionality, provide detailed test scenarios.

#### **Scenario 1: Basic shell commands**
- **Role Involved:** For every role
- **Objective:** interact with the file system, read, write, move, rename, delete a file or folder.
- **Preconditions:** Installing Wormhole
- **Test Steps:**
  1. Create a file
  2. Create a folder
  3. Write to a file
  4. Read the file
  5. Create files into a folder
  6. Create folders into a folder
  7. Move a file into an other folder
  8. Move a folder into an other folder
  9. Rename a file
  10. Rename a folder
  11. Delete a file
  12. Delete a folder
  13. List the content of a folder
  14. Get metadata of a file or folder
- **Expected Outcome:** If all this commands work correctly, the user should be able to interact with the file system.

#### **Scenario 2: Docker Image**
- **Role Involved:** Wormhole Developper
- **Objective:** Testing wormhole into a docker image
- **Preconditions:** Installing Wormhole and Docker
- **Test Steps:**
  1. Retrieve the Wormhole Docker image from the official repository.
  2. Launch the Docker container with the appropriate command.
  3. Check that Wormhole starts correctly by consulting the logs or the container status.
  4. Mount the Wormhole file system in the container.
  5. Perform basic operations: create, write, read and delete files.
  6. Check for errors or unexpected behavior.
- **Expected Outcome:** The Docker image runs Wormhole correctly, and all basic file system operations run smoothly.

#### **Scenario 3: Configuration files**
- **Role Involved:** For every role
- **Objective:** Create file configuration, based on the documentation and use it.
- **Preconditions:** Installing Wormhole
- **Test Steps:**
  1. Use the Wormhole CLI to generate a default configuration file.
  2. Modify a specific option (e.g. mount point or redundancy parameters).
  3. Start Wormhole with the modified configuration and check that the changes are applied.
  4. Modify another option (e.g. cache settings).
  5. Reload configuration using CLI without restarting Wormhole.
  6. Check that the new configuration has been taken into account (e.g. change in cache behavior).
- **Expected Outcome:** Documentation is clear and easy to use. The configuration is really corectly used before and after the reload.

#### **Scenario 4: Stability**
- **Role Involved:** For every role
- **Objective:** Test Wormhole's stability under various operations without crashes or critical bugs.
- **Preconditions:** Installing Wormhole and configure a basic instance
- **Test Steps:**
  1. Perform a rapid series of operations: create several files, write, read, delete, rename.
  2. Attempt operations likely to generate errors (e.g. deleting a non-existent file).
  3. Run Wormhole for 24 hours with periodic operations.
  4. Upload and download large files (e.g. 5GB).
  5. Monitor logs for errors or warnings.
  6. Interrupt Wormhole (e.g. sudden stop) and restart to verify recovery.
- **Expected Outcome:** Wormhole manages all operations without crashing, logs errors appropriately and maintains data integrity.

#### **Scenario 5: Linux Support**
- **Role Involved:** Home server user
- **Objective:** Testing if Wormhole works on Linux systems (Debian, Ubuntu, etc.)
- **Preconditions:** Installing Wormhole
- **Test Steps:**
  1. Installer Wormhole sur un système basé sur Debian (ex. Ubuntu).
  2. Effectuer des opérations de base : créer, lire, écrire, supprimer des fichiers et dossiers.
  3. Tester des fonctionnalités avancées (ex. fichiers de configuration, redondance si applicable).
  4. Répéter l’installation et les tests sur une autre distribution (ex. Fedora).
  5. Vérifier l’absence de problèmes spécifiques à une distribution.
- **Expected Outcome:** Wormhole installs and runs correctly on several Linux distributions, with all basic features operational.

#### **Scenario 6: Mac Support**
- **Role Involved:** Mac user
- **Objective:** Testing if Wormhole works on Mac systems.
- **Preconditions:** Installing Wormhole
- **Test Steps:**
  1. Installer Wormhole sur un système macOS.
  2. Effectuer des opérations de base : créer, lire, écrire, supprimer des fichiers et dossiers.
  3. Vérifier les problèmes spécifiques à macOS (ex. permissions, intégration avec Finder).
  4. Tester l’interface utilisateur pour s’assurer qu’elle est intuitive pour les utilisateurs Mac.
- **Expected Outcome:** Wormhole installs and runs on macOS with the correct basic operations, although limitations may exist due to low priority.

#### **Scenario 7: Window Support**
- **Role Involved:** Windows user
- **Objective:** Testing if Wormhole works on Windows systems.
- **Preconditions:** Installing Wormhole
- **Test Steps:**
  1. Install Wormhole on a Windows system (e.g. Windows 10 or 11).
  2. Perform basic operations: create, read, write, delete files and folders.
  3. Test Windows-specific integrations (e.g. mount as drive letter).
  4. Identify unimplemented or buggy features (linked to 70% status).
- **Expected Outcome:** Wormhole installs and runs on Windows, with most basic features operational, but possible limitations or bugs to be documented.

#### **Scenario 8: Complete User Documentation**
- **Role Involved:** For ervery role
- **Objective:** Check if the documentation is clear and easy to use for the user.
- **Preconditions:** No required setup
- **Test Steps:**
  1. Read the documentation for installing Wormhole
  2. Read the documentation for using Wormhole
  3. Read the documentation for configuring Wormhole
  4. Identify incomplete or unclear sections.
  5. Suggest improvements or additions to the content.
- **Expected Outcome:** If the documentation is clear and easy to use, the user should be able to install, use and configure Wormhole.

#### **Scenario 9: Redudancy**
- **Role Involved:** For every role
- **Objective:** Testing the redundancy of the system.
- **Preconditions:** Installing Wormhole on 3 different machines
- **Test Steps:**
  1. Create a Wormhole instance on each machine in the same Wormhole network.
  2. Create a different file on each machine.
  3. Check that the files are available on all machines.
  4. Shut down machine *A*.
  5. Check that the files remain available on the remaining machines (machine *B* must retrieve the file from *A*).
  6. Modify machine *A*'s file on machine *B*.
  7. Restart machine *A*.
  8. Check that the modified file is updated on machine *A*.
- **Expected Outcome:** The system maintains file availability and consistency, even in the event of a node failure, and changes are propagated correctly.

#### **Scenario 10: Clean Error**
- **Role Involved:** For every role
- **Objective:** Testing the error handling of the system are complete and understandable by the user.
- **Preconditions:** Create a Wormhole network
- **Test Steps:**
  1. Attempt an operation that should fail (e.g. write to a read-only file or access a non-existent file).
  2. Observe the error message or feedback provided by Wormhole.
  3. Check that the message is clear and helps to understand the problem and its resolution.
  4. Test error handling in different contexts.
  5. Check that errors are logged for debugging purposes.
- **Expected Outcome:** Wormhole provides clear and useful error messages that facilitate troubleshooting without confusing the user.
---

## **3. Success Criteria**
[Define the metrics and conditions that determine if the beta version is successful.]

The following criteria will be used to determine the success of the beta version.

| **Criterion** | **Description**                                                                          | **Threshold for Success**               |
| ------------- | ---------------------------------------------------------------------------------------- | --------------------------------------- |
| Stability     | No major crashes or critical bugs                                                        | No crash reported                       |
| Usability     | Users can use like a Google Drive                                                        | 80% positive feedback from testers      |
| Performance   | 95% of file operations (create, read, synchronize) are completed in less than 10 seconds | 95% of files analyzed within 10 seconds |
| Accuracy      | 90 % des cas de test réussissent sans erreurs                                            | 90% accuracy in test cases              |

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