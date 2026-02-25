# Bevy GAS Development Composite Skill Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Create a composite skill that integrates Bevy ECS, bevy_gameplay_tag, and gameplay-abilities-system-rust-ecs knowledge for automatic context-aware assistance.

**Architecture:** Single markdown file containing all three skills' content, organized into modules with clear boundaries. Auto-triggers when detecting Bevy + GAS project context via Cargo.toml dependencies.

**Tech Stack:** Markdown, Claude Code skill system, @superpowers:writing-skills

---

## Task 1: Locate and Read Source Skills

**Files:**
- Read: `~/.claude/skills/bevy/SKILL.md` (or plugin cache location)
- Read: `~/.claude/skills/bevy-gameplay-tag/SKILL.md`
- Read: `~/.claude/skills/gameplay-abilities-system-rust-ecs/SKILL.md`

**Step 1: Find skills directory structure**

Run: `find ~/.claude -type d -name "skills" 2>/dev/null | head -5`
Expected: List of skill directories

**Step 2: Locate bevy skill**

Run: `find ~/.claude -name "bevy.md" -o -name "SKILL.md" | grep -i bevy | head -10`
Expected: Path to bevy skill file

**Step 3: Read bevy skill content**

Use Read tool on the located file path.
Expected: Full content of Bevy ECS skill

**Step 4: Locate bevy-gameplay-tag skill**

Run: `find ~/.claude -path "*/bevy-gameplay-tag/*" -name "*.md" | head -10`
Expected: Path to bevy-gameplay-tag skill file

**Step 5: Read bevy-gameplay-tag skill content**

Use Read tool on the located file path.
Expected: Full content of gameplay tag skill

**Step 6: Locate gameplay-abilities-system-rust-ecs skill**

Run: `find ~/.claude -path "*gameplay-abilities*" -name "*.md" | head -10`
Expected: Path to GAS skill file

**Step 7: Read gameplay-abilities-system-rust-ecs skill content**

Use Read tool on the located file path.
Expected: Full content of GAS skill

**Step 8: Document findings**

Create a temporary note with file paths and content summaries.
Expected: Clear reference for next tasks

---

## Task 2: Create Skill Directory Structure

**Files:**
- Create: `~/.claude/skills/bevy-gas-development/SKILL.md`

**Step 1: Determine skill installation location**

Run: `ls -la ~/.claude/skills/ 2>/dev/null || echo "Using plugin cache"`
Expected: Either user skills dir exists or need to use plugin location

**Step 2: Create skill directory**

Run: `mkdir -p ~/.claude/skills/bevy-gas-development`
Expected: Directory created successfully

**Step 3: Verify directory creation**

Run: `ls -la ~/.claude/skills/bevy-gas-development/`
Expected: Empty directory exists

---

## Task 3: Write Skill Header and Metadata

**Files:**
- Create: `~/.claude/skills/bevy-gas-development/SKILL.md`

**Step 1: Write skill header**

```markdown
# Bevy GAS Development Guide

<!-- Source: Composite skill combining bevy, bevy-gameplay-tag, and gameplay-abilities-system-rust-ecs -->
<!-- Maintained: 2026-02-25 -->

Comprehensive guide for Bevy game development with GameplayAbilitySystem (GAS). Combines Bevy ECS patterns, bevy_gameplay_tag hierarchical tag system, and complete GAS implementation (Attributes, GameplayEffects, GameplayAbilities, GameplayCues). Use when working on Bevy projects that implement gameplay systems, ability mechanics, tag-based logic, or attribute/effect systems.

## Auto-Trigger Conditions

This skill automatically activates when:
1. `Cargo.toml` contains `bevy` dependency (any version)
2. AND one of:
   - Contains `bevy_gameplay_tag` dependency
   - Contains `bevy_gameplay_ability_system` dependency
   - Working directory path contains "bevy_gameplay_ability_system"

---
```

**Step 2: Create the file with header**

Use Write tool to create `~/.claude/skills/bevy-gas-development/SKILL.md` with the header content.
Expected: File created with metadata

**Step 3: Verify file creation**

Run: `head -20 ~/.claude/skills/bevy-gas-development/SKILL.md`
Expected: Header content displayed correctly

---

## Task 4: Add Bevy ECS Module

**Files:**
- Modify: `~/.claude/skills/bevy-gas-development/SKILL.md`

**Step 1: Extract Bevy ECS core content**

From the bevy skill content read in Task 1, extract:
- ECS architecture principles
- Component design patterns
- System execution ordering
- Query optimization
- UI development patterns
- Common pitfalls

**Step 2: Append Bevy module section**

```markdown
## Module 1: Bevy ECS Core Patterns

<!-- Source: bevy skill -->

### ECS Architecture Principles

[Insert extracted Bevy ECS architecture content here]

### Component Design Patterns

[Insert extracted component design content here]

### System Execution Order

[Insert extracted system ordering content here]

### Query Optimization

[Insert extracted query optimization content here]

### UI Development

[Insert extracted UI patterns content here]

### Common Pitfalls

[Insert extracted common pitfalls content here]

---
```

**Step 3: Append to skill file**

Use Edit tool to append the Bevy module content.
Expected: Bevy module added to skill file

**Step 4: Verify content**

Run: `grep -n "Module 1: Bevy ECS" ~/.claude/skills/bevy-gas-development/SKILL.md`
Expected: Line number showing module was added

---

## Task 5: Add Gameplay Tags Module

**Files:**
- Modify: `~/.claude/skills/bevy-gas-development/SKILL.md`

**Step 1: Extract gameplay tags content**

From the bevy-gameplay-tag skill content read in Task 1, extract:
- Hierarchical tag system concepts
- GameplayTagContainer API
- Tag matching rules (Exact/Partial/Any/All)
- Tag counting mechanism
- Tag event system

**Step 2: Append gameplay tags module section**

```markdown
## Module 2: Gameplay Tags System

<!-- Source: bevy-gameplay-tag skill -->

### Hierarchical Tag System

[Insert extracted tag hierarchy content here]

### GameplayTagContainer Usage

[Insert extracted container API content here]

### Tag Matching Rules

[Insert extracted matching rules content here]

### Tag Counting

[Insert extracted tag counting content here]

### Tag Events

[Insert extracted tag events content here]

---
```

**Step 3: Append to skill file**

Use Edit tool to append the gameplay tags module content.
Expected: Gameplay tags module added to skill file

**Step 4: Verify content**

Run: `grep -n "Module 2: Gameplay Tags" ~/.claude/skills/bevy-gas-development/SKILL.md`
Expected: Line number showing module was added

---

## Task 6: Add GameplayAbilitySystem Module - Attributes

**Files:**
- Modify: `~/.claude/skills/bevy-gas-development/SKILL.md`

**Step 1: Extract attributes system content**

From the gameplay-abilities-system-rust-ecs skill content read in Task 1, extract:
- Dual-value model (BaseValue/CurrentValue)
- AttributeSetDefinition trait
- Modifier application order (Add → Multiply → Override)
- Entity-per-attribute pattern
- AttributeOwner component

**Step 2: Append attributes subsection**

```markdown
## Module 3: GameplayAbilitySystem Implementation

<!-- Source: gameplay-abilities-system-rust-ecs skill -->

### Attributes System

#### Dual-Value Model

[Insert extracted dual-value model content here]

#### AttributeSetDefinition Trait

[Insert extracted trait definition content here]

#### Modifier Application Order

[Insert extracted modifier order content here]

#### Entity-Per-Attribute Pattern

[Insert extracted entity pattern content here]

---
```

**Step 3: Append to skill file**

Use Edit tool to append the attributes subsection.
Expected: Attributes subsection added

**Step 4: Verify content**

Run: `grep -n "Attributes System" ~/.claude/skills/bevy-gas-development/SKILL.md`
Expected: Line number showing subsection was added

---

## Task 7: Add GameplayAbilitySystem Module - Effects

**Files:**
- Modify: `~/.claude/skills/bevy-gas-development/SKILL.md`

**Step 1: Extract effects system content**

From the gameplay-abilities-system-rust-ecs skill, extract:
- GameplayEffectDefinition and registry pattern
- Duration policies (Instant/HasDuration/Infinite)
- Periodic execution
- Stacking mechanisms (Independent/RefreshDuration/StackCount)
- Tag requirements for application

**Step 2: Append effects subsection**

```markdown
### GameplayEffects System

#### Definition and Registry Pattern

[Insert extracted definition pattern content here]

#### Duration Policies

[Insert extracted duration policies content here]

#### Periodic Execution

[Insert extracted periodic execution content here]

#### Stacking Mechanisms

[Insert extracted stacking content here]

#### Tag Requirements

[Insert extracted tag requirements content here]

---
```

**Step 3: Append to skill file**

Use Edit tool to append the effects subsection.
Expected: Effects subsection added

**Step 4: Verify content**

Run: `grep -n "GameplayEffects System" ~/.claude/skills/bevy-gas-development/SKILL.md`
Expected: Line number showing subsection was added

---

## Task 8: Add GameplayAbilitySystem Module - Abilities

**Files:**
- Modify: `~/.claude/skills/bevy-gas-development/SKILL.md`

**Step 1: Extract abilities system content**

From the gameplay-abilities-system-rust-ecs skill, extract:
- AbilityDefinition and registry pattern
- Activation flow (TryActivate → Commit → End/Cancel)
- Cost and cooldown mechanics
- Tag requirements, blocking, and cancellation

**Step 2: Append abilities subsection**

```markdown
### GameplayAbilities System

#### Definition and Registry Pattern

[Insert extracted definition pattern content here]

#### Activation Flow

[Insert extracted activation flow content here]

#### Costs and Cooldowns

[Insert extracted cost/cooldown content here]

#### Tag Requirements and Blocking

[Insert extracted tag mechanics content here]

---
```

**Step 3: Append to skill file**

Use Edit tool to append the abilities subsection.
Expected: Abilities subsection added

**Step 4: Verify content**

Run: `grep -n "GameplayAbilities System" ~/.claude/skills/bevy-gas-development/SKILL.md`
Expected: Line number showing subsection was added

---

## Task 9: Add GameplayAbilitySystem Module - Cues

**Files:**
- Modify: `~/.claude/skills/bevy-gas-development/SKILL.md`

**Step 1: Extract cues system content**

From the gameplay-abilities-system-rust-ecs skill, extract:
- GameplayCueManager resource
- Static handlers (trait-based, no entity)
- Actor handlers (spawned entity)
- Hierarchical tag matching for cue routing

**Step 2: Append cues subsection**

```markdown
### GameplayCues System

#### GameplayCueManager

[Insert extracted manager content here]

#### Static Handlers

[Insert extracted static handler content here]

#### Actor Handlers

[Insert extracted actor handler content here]

#### Hierarchical Tag Matching

[Insert extracted tag matching content here]

---
```

**Step 3: Append to skill file**

Use Edit tool to append the cues subsection.
Expected: Cues subsection added

**Step 4: Verify content**

Run: `grep -n "GameplayCues System" ~/.claude/skills/bevy-gas-development/SKILL.md`
Expected: Line number showing subsection was added

---

## Task 10: Add Integration Guide

**Files:**
- Modify: `~/.claude/skills/bevy-gas-development/SKILL.md`

**Step 1: Write integration patterns section**

```markdown
## Integration Guide and Best Practices

### System Execution Order

All GAS systems run in `Update` schedule, chained via `GasSystemSet`:

```
Input → Attributes → Effects → Abilities → Cues → Cleanup
```

Each top-level set has sub-sets. Add custom systems with `.in_set(GasSystemSet::X)`.

### Entity-Per-Thing Pattern

Attributes, effects, and abilities are separate entities (not Vec on owner). This enables:
- Bevy query optimization
- Parallel execution
- Clean component composition

### Event-Driven Communication

Cross-module communication uses Bevy events:
- `ApplyGameplayEffectEvent` - Apply effect to target
- `TryActivateAbilityEvent` - Request ability activation
- `GameplayCueEvent` - Trigger visual/audio feedback

### Using Tags in Abilities

```rust
// In ability definition
AbilityDefinition::new("fireball")
    .with_activation_required_tags(tags_manager, &["Ability.Castable"])
    .with_activation_blocked_tags(tags_manager, &["Status.Silenced"])
    .with_cancel_tags(tags_manager, &["Status.Stunned"])
```

### Querying Attributes in Effects

```rust
// In effect modifier
fn apply_damage_based_on_strength(
    mut attributes: Query<&mut Attribute>,
    owners: Query<&AttributeOwner>,
) {
    // Query owner's strength attribute
    // Apply damage modifier based on strength value
}
```

### Testing Patterns

Tests use `App::new()` with necessary plugins:

```rust
#[test]
fn test_ability_activation() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        GameplayTagsPlugin::with_data_path("assets/gameplay_tags.json".to_string()),
        AbilitiesPlugin,
    ));
    app.update(); // Load tags

    app.world_mut().run_system_once(|mut commands: Commands| {
        // Test logic here
    });
}
```

### Code Quality Principles

- **Correctness over convenience** - Crash on invalid state
- **Make illegal states unrepresentable** - Use enums over strings
- **Exhaustive pattern matching** - No wildcards
- **Document WHY, not what** - Explain reasoning
- **No over-engineering** - Only requested changes
- **Delete unused code completely** - No backwards-compat hacks

---
```

**Step 2: Append integration guide**

Use Edit tool to append the integration guide section.
Expected: Integration guide added to skill file

**Step 3: Verify content**

Run: `grep -n "Integration Guide" ~/.claude/skills/bevy-gas-development/SKILL.md`
Expected: Line number showing section was added

---

## Task 11: Add Usage Examples

**Files:**
- Modify: `~/.claude/skills/bevy-gas-development/SKILL.md`

**Step 1: Write complete usage example**

```markdown
## Complete Usage Example

### Scenario: Fireball Ability with Mana Cost

```rust
// 1. Define attributes
#[derive(Component)]
struct ManaAttribute;

impl AttributeSetDefinition for ManaAttribute {
    fn attribute_name() -> &'static str { "Mana" }
}

// 2. Define effect (mana cost)
let mana_cost_effect = GameplayEffectDefinition::new("mana_cost")
    .with_duration_policy(DurationPolicy::Instant)
    .add_modifier(
        AttributeModifier::new("Mana")
            .with_operation(ModifierOperation::Add)
            .with_magnitude(-25.0)
    );

// 3. Define ability
let fireball_ability = AbilityDefinition::new("fireball")
    .with_activation_required_tags(tags_manager, &["Ability.Castable"])
    .with_activation_blocked_tags(tags_manager, &["Status.Silenced"])
    .with_cost_effect("mana_cost")
    .with_cooldown(5.0);

// 4. Register definitions
effect_registry.register(mana_cost_effect);
ability_registry.register(fireball_ability);

// 5. Grant ability to entity
commands.entity(player).insert((
    AbilitySpec::new("fireball"),
    AbilityOwner(player),
));

// 6. Activate ability
event_writer.send(TryActivateAbilityEvent {
    ability: ability_entity,
    instigator: player,
});

// 7. Handle cue for visual feedback
#[derive(Component)]
struct FireballCueHandler;

impl GameplayCueHandler for FireballCueHandler {
    fn handle_cue(&self, cue_event: &GameplayCueEvent, commands: &mut Commands) {
        // Spawn fireball particle effect
    }
}
```

---
```

**Step 2: Append usage example**

Use Edit tool to append the usage example section.
Expected: Usage example added to skill file

**Step 3: Verify content**

Run: `grep -n "Complete Usage Example" ~/.claude/skills/bevy-gas-development/SKILL.md`
Expected: Line number showing section was added

---

## Task 12: Verify and Test Skill

**Files:**
- Read: `~/.claude/skills/bevy-gas-development/SKILL.md`

**Step 1: Read complete skill file**

Use Read tool to review the entire skill file.
Expected: All sections present and properly formatted

**Step 2: Check file size**

Run: `wc -l ~/.claude/skills/bevy-gas-development/SKILL.md`
Expected: Reasonable line count (likely 1000-3000 lines)

**Step 3: Verify markdown syntax**

Run: `grep -E "^#{1,6} " ~/.claude/skills/bevy-gas-development/SKILL.md | head -20`
Expected: Proper heading hierarchy

**Step 4: Check for source attribution**

Run: `grep -n "Source:" ~/.claude/skills/bevy-gas-development/SKILL.md`
Expected: Source comments present for each module

**Step 5: Commit the skill file**

```bash
git add docs/plans/2026-02-25-bevy-gas-development-skill.md
git commit -m "feat: add bevy-gas-development composite skill implementation plan"
```

---

## Task 13: Document Installation Instructions

**Files:**
- Create: `~/.claude/skills/bevy-gas-development/README.md`

**Step 1: Write README**

```markdown
# Bevy GAS Development Skill

Composite skill for Bevy game development with GameplayAbilitySystem.

## Installation

This skill is located at: `~/.claude/skills/bevy-gas-development/SKILL.md`

## Auto-Trigger

Automatically activates when:
1. `Cargo.toml` contains `bevy` dependency
2. AND one of:
   - Contains `bevy_gameplay_tag` dependency
   - Contains `bevy_gameplay_ability_system` dependency
   - Working directory contains "bevy_gameplay_ability_system"

## Manual Invocation

Use: `/bevy-gas-development` or reference with `@bevy-gas-development`

## Contents

- Bevy ECS core patterns
- bevy_gameplay_tag hierarchical tag system
- Complete GameplayAbilitySystem implementation
- Integration guide and best practices

## Maintenance

Source skills:
- `bevy` - ECS architecture and patterns
- `bevy-gameplay-tag` - Gameplay tag system
- `gameplay-abilities-system-rust-ecs` - GAS implementation

When source skills update, sync corresponding sections in this composite skill.

## Version

Created: 2026-02-25
Last Updated: 2026-02-25
```

**Step 2: Create README file**

Use Write tool to create the README.
Expected: README created successfully

**Step 3: Verify README**

Run: `cat ~/.claude/skills/bevy-gas-development/README.md`
Expected: README content displayed

---

## Verification

After completing all tasks:

1. Skill file exists at `~/.claude/skills/bevy-gas-development/SKILL.md`
2. File contains all three source skills' content
3. Content is organized into clear modules
4. Integration guide provides cross-module examples
5. Source attribution comments present
6. README documents installation and usage

## Notes

- Use @superpowers:writing-skills for skill-specific guidance
- Reference design doc at `docs/plans/2026-02-25-composite-skill-design.md`
- Test auto-trigger by opening this project in a new Claude Code session
