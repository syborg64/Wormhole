---
title: "KPI Report"  
project: "Wormhole"  
team: "Axel Denis, Julian Scott, Ludovic de Chavagnac, Arthur Aillet"  
mentor: "Alexandre Pereira"  
period: "From 01/07/2025 to 12/07/2025"  
---

# 1. Context and Summary  
- **Objective of the Document**: To present the progress of track-specific KPIs (Technical) for the Wormhole project, which aims to develop a decentralized data storage solution with a mesh architecture.  
- **Period Covered**: From 01/07/2025 to 12/07/2025.  
- **Version of the Report**: v1.0  

---

# 2. KPI Tracking Table  

| KPI (SMART Objective)                              | Period          | Target                        | Current Value   | Evidence                          | Progress Analysis                          | Next Step                                |
|----------------------------------------------------|-----------------|-------------------------------|-----------------|-----------------------------------|--------------------------------------------|------------------------------------------|
| Develop 3 functional prototypes by 30/09/2025   (exemple)   | 01/06/2024 – 30/09/2025   | 3 prototypes                  | 1               | •     | 33% achieved. Initial Rust prototype completed, but pace needs to increase. | • Finalize v2 and v3 by end of August.   |

---

# 3. Glossary & Methodology  

- **KPI Definitions**:  
  - *Develop 3 functional prototypes*: Number of working versions of the Wormhole system built and tested (e.g., Rust-based prototype).  
  - *Integrate FUSE and WinFsp*: Successful integration of Filesystem in Userspace (Linux) and Windows File System Proxy (Windows) into the prototype.  
  - *Conduct 2 user feedback sessions*: Number of sessions with external stakeholders (e.g., BPCE, Grant Thornton) to gather feedback.  
- **Data Sources**: Rust codebase (GitHub commits), integration logs.  

---

# 4. Visualization of Progress  

> *(Insert a chart here showing the evolution of each KPI against its target over time)*  

![KPI Evolution](path/to/chart-kpis.png)  

---

# 5. Evidence (Tangible Proof)  

| Evidence Type                 | Description                         | Link / Path                           |
|-------------------------------|-------------------------------------|---------------------------------------|
| Prototype Screenshot          | Rust-based prototype v1             | `evidence/proto-v1-rust.png`          |
| Commit Log                    | Initial prototype commit            | `evidence/commit-proto-v1.txt`        |

---

# 6. Comments, Risks & Corrective Actions  

- **Comments**: The first prototype in Rust is functional and aligns with the "security first" approach, but integration and user feedback are lagging.  
- **Risks**:  
  - Delay in FUSE/WinFsp integration may hinder cross-platform testing.  
  - Lack of user feedback could misalign the solution with enterprise needs (e.g., simplicity, security).  
- **Corrective Actions**:  
  - Prioritize FUSE integration in July.  
  - Contact BPCE or Grant Thornton contacts (e.g., Nicolas Storez) for feedback sessions.  

---

# 7. Action Plan / Roadmap  

| Objective                  | Action to Take                  | Responsible     | Deadline     |
|----------------------------|---------------------------------|-----------------|--------------|
| Complete 3 prototypes      | Develop v2 and v3 prototypes    | Axel            | 30/04/2025   |
| Integrate WinFsp             | Implement FUSE in prototype     | Julian          | 20/07/2025   |