Domain Model: Stellar Outpost

Entities:

- Outpost: Represents the player's space station outpost.
- Module: Individual modules that make up the outpost, such as living quarters, research labs, power generators, etc.
- Crew: Crew members with unique skills, personalities, and roles.
- Resource: Various resources consumed and managed within the outpost, including energy, food, water, and minerals.
- Expedition: Missions sent out from the outpost to explore the galaxy and encounter various events and encounters.
- Upgrade: Technological advancements and improvements that enhance the outpost's capabilities and functionalities.
- Threat: Challenges and dangers that pose risks to the outpost, such as asteroid showers, alien invasions, power outages, etc.

Actions/Behaviors:

- Construct: Build new modules to expand the outpost.
- Allocate Resources: Distribute resources to different modules and crew members.
- Recruit: Hire new crew members with specific skills and attributes.
- Assign Tasks: Assign crew members to different tasks and roles within the outpost.
- Train: Improve crew members' skills and abilities through training and specialization.
- Explore: Initiate expeditions to explore the procedurally generated galaxy.
- Encounter: Experience various events, encounters, and challenges during expeditions.
- Manage Threats: Respond to threats and crises by making decisions and taking appropriate actions.
- Research: Conduct research to unlock new technologies, upgrades, and blueprints.
- Upgrade: Enhance outpost infrastructure, defenses, and spacecraft capabilities.
- Permadeath: Handle permanent consequences and learning from failures in each run.

Relationships:

- Outpost has Modules: An outpost consists of multiple interconnected modules that form its infrastructure.
- Outpost has Crew: Crew members reside and perform tasks within the outpost.
- Crew interacts with Crew: Crew members interact with each other, forming relationships and influencing morale.
- Resource is Consumed by Modules and Crew: Modules and crew members consume various resources to meet their needs.
- Expedition is Initiated by Outpost: The outpost initiates expeditions to explore the galaxy.
- Crew Participates in Expeditions: Selected crew members form an expedition team and embark on exploration missions.
- Expedition Encounters Threats and Events: Expeditions encounter various events, challenges, and potential threats.
- Upgrade Unlocks New Technologies: Researching and acquiring upgrades unlocks new technologies and capabilities for the outpost.
- Upgrades Enhance Modules and Outpost: Upgrades improve module functionalities, outpost infrastructure, and defenses.
