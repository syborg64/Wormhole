# **BETA TEST PLAN – TIDYMASTER**

## **1. Core Functionalities for Beta Version**
Below are the essential features that must be available for beta testing, along with any changes made since the initial Tech3 Action Plan.

| **Feature Name**  | **Description** | **Priority (High/Medium/Low)** | **Changes Since Tech3** |
|-------------------|---------------|--------------------------------|--------------------------|
| File Analysis Engine  | Automatically scans stored files and assigns a **Tidyscore** based on naming conventions, duplicates, and inactivity. | High | Refined analysis criteria. |
| User Dashboard  | Provides users with an overview of their storage efficiency, Tidyscore breakdown, and optimization recommendations. | High | - |
| Duplicate Detection  | Identifies and groups duplicate files for easier deletion or consolidation. | High | - |
| Automated Recommendations  | Suggests actions (deletion, archiving, renaming) based on Tidyscore and user-defined rules. | Medium | - |
| Multi-User Access Control  | Allows different user roles (Admin, Employee) with appropriate permissions. | Medium | Role-based permissions refined. |

---

## **2. Beta Testing Scenarios**

### **2.1 User Roles**
The following roles will be involved in beta testing.

| **Role Name**  | **Description** |
|---------------|---------------|
| Admin        | Manages storage settings, accesses all analytics, and applies global storage policies. |
| Employee     | Views personal storage insights and receives file optimization recommendations. |

---

### **2.2 Test Scenarios**
For each core functionality, detailed test scenarios are provided below.

#### **Scenario 1: File Analysis Engine**
- **Role Involved:** Admin / Employee
- **Objective:** Ensure the Tidyscore is correctly calculated and displayed.
- **Preconditions:** A set of sample files with various naming conventions, duplicates, and different last-modified dates.
- **Test Steps:**
  1. Upload test files into the system.
  2. Trigger the analysis process.
  3. Verify that each file is assigned a Tidyscore.
  4. Ensure that scores are correctly based on naming conventions, duplicates, and inactivity.
- **Expected Outcome:** Each file receives a correct and justified Tidyscore.

#### **Scenario 2: User Dashboard**
- **Role Involved:** Admin / Employee
- **Objective:** Ensure that users can view their Tidyscore summary and recommendations.
- **Preconditions:** Tidyscore has already been calculated for existing files.
- **Test Steps:**
  1. Log in as a user.
  2. Navigate to the dashboard.
  3. Verify that all relevant analytics (Tidyscore, file health, duplicate alerts) are displayed.
  4. Check that clicking on a recommendation provides relevant details.
- **Expected Outcome:** Users can see their storage insights and interact with optimization recommendations.

#### **Scenario 3: Duplicate Detection**
- **Role Involved:** Admin
- **Objective:** Ensure the system detects duplicate files accurately.
- **Preconditions:** A set of duplicate files uploaded to storage.
- **Test Steps:**
  1. Upload a mix of original and duplicate files.
  2. Trigger the duplicate detection function.
  3. Verify that all duplicates are correctly identified.
  4. Test the deletion or consolidation options.
- **Expected Outcome:** Duplicates are identified and handled appropriately.

#### **Scenario 4: Multi-User Access Control**
- **Role Involved:** Admin / Employee
- **Objective:** Ensure that different user roles have the correct permissions.
- **Preconditions:** A system with at least one Admin and one Employee account.
- **Test Steps:**
  1. Log in as an Admin and attempt to modify system-wide settings.
  2. Log in as an Employee and attempt to modify system-wide settings.
  3. Verify that only Admins can perform administrative actions.
- **Expected Outcome:** Role-based restrictions function as intended.

---

## **3. Success Criteria**
The following criteria will be used to determine the success of the beta version.

| **Criterion** | **Description** | **Threshold for Success** |
|--------------|---------------|------------------------|
| Stability    | No major crashes or critical bugs | No crash reported |
| Usability    | Users can navigate and understand features with minimal guidance | 80% positive feedback from testers |
| Performance  | File analysis completes within an acceptable time frame | 95% of files analyzed within 10 seconds |
| Accuracy    | Tidyscore and duplicate detection provide reliable results | 90% accuracy in test cases |

---

## **4. Known Issues & Limitations**

| **Issue** | **Description** | **Impact** | **Planned Fix? (Yes/No)** |
|----------|---------------|----------|----------------|
| UI Lag  | Some dashboard elements take longer to load with large data sets. | Medium | Yes |
| False Positives in Duplicate Detection | Some similar files are incorrectly marked as duplicates. | High | Yes |
| Tidyscore Calculation Delay | Large storage spaces take longer to analyze. | Medium | No (Planned for post-beta fix) |

---

## **5. Conclusion**
This Beta Test Plan ensures that **TidyMaster** is tested in a structured and efficient manner. By validating the core functionalities, refining the user experience, and ensuring feature stability, we aim to optimize data storage management for businesses. The insights from this beta phase will help us address key issues and prepare for the final version of the project.