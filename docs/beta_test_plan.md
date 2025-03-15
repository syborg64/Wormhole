# **WormHole:** Beta Test Plan

## Core functionalities necessary for a proper beta test:

- Read, Write, Move, Rename, Delete files and folders (done)
    - All common user interactions must be implemented
- Docker Image (done)
    - Necessary to simplify the user interaction
- Configuration files (done)
- Stability (Almost)
    - Most of the Fuser Calls must be implemented to allow the best interaction with interfaces
- Linux Support (done)
- Mac Support (lack testing)
- Windows Support (70%)
- Complete User Documentation (50%)
- Redudancy (Just Started)
    - Needed to justify wormhole usage
- Clean Error Handling
    - Mostly only used by TUI but will help with feedback

## Currently aborted features:

- Fancy optimization techniques
    - Trafic optimization depending on internet speed or storage left
- Pod specialisation
    - Permissions
    - Storage Limit
    - Redudancy priority
- Cache handling

## Profiles

### An experienced mac user
> Mac support, ease of use

Allow us to have some insight on the mac experience, bugs unique to mac

A mac user is also used to easy to use interfaces so we might have usefull API feedback

### A home server user
> Large Dataset, speed test

A home server user with a large catalogue of files, so we can have a real insight of how our system handle realistic high work load.

We could do speed tests and comporate to other filesystems to verify efficiency.

### Professional engineer
> API Feedback from experienced user, Point of vue from a professional

Working with a professional will allow us to have feedback with the intrest of company in mind.

It will also let us work with someone with professional experience so a wider skill set.
