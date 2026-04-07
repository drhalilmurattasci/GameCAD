//! Command pattern implementation providing undo/redo support.
//!
//! Each undoable action implements the [`Command`] trait. Commands are executed
//! through [`CommandHistory`] which maintains undo and redo stacks and emits
//! events via the [`EventBus`].

use anyhow::Result;

use crate::ecs::World;
use crate::events::{EventBus, RedoPerformed, UndoPerformed};

// ─────────────────────────────────────────────────────────────────────
// Command trait
// ─────────────────────────────────────────────────────────────────────

/// A reversible editor operation.
pub trait Command: Send + Sync {
    /// Executes the command, applying its effect to the world.
    fn execute(&mut self, ctx: &mut CommandContext) -> Result<()>;

    /// Reverses the effect of a previous [`execute`](Command::execute) call.
    fn undo(&mut self, ctx: &mut CommandContext) -> Result<()>;

    /// A human-readable description of the command (shown in the Edit menu).
    fn description(&self) -> &str;
}

// ─────────────────────────────────────────────────────────────────────
// CommandContext
// ─────────────────────────────────────────────────────────────────────

/// Provides mutable access to the systems a [`Command`] typically needs.
pub struct CommandContext<'a> {
    /// The ECS world.
    pub world: &'a mut World,
    /// The global event bus.
    pub events: &'a EventBus,
}

impl<'a> CommandContext<'a> {
    /// Creates a new context.
    pub fn new(world: &'a mut World, events: &'a EventBus) -> Self {
        Self { world, events }
    }
}

// ─────────────────────────────────────────────────────────────────────
// CommandHistory
// ─────────────────────────────────────────────────────────────────────

/// Manages the undo and redo stacks for executed [`Command`]s.
pub struct CommandHistory {
    undo_stack: Vec<Box<dyn Command>>,
    redo_stack: Vec<Box<dyn Command>>,
    /// Maximum number of commands to keep in the undo stack.
    pub max_depth: usize,
}

impl Default for CommandHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandHistory {
    /// Default maximum undo depth.
    pub const DEFAULT_MAX_DEPTH: usize = 100;

    /// Creates a new, empty history.
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_depth: Self::DEFAULT_MAX_DEPTH,
        }
    }

    /// Creates a history with a custom max depth.
    pub fn with_max_depth(max_depth: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_depth,
        }
    }

    /// Executes a command and pushes it onto the undo stack.
    ///
    /// The redo stack is cleared since the history has diverged.
    pub fn execute(
        &mut self,
        mut cmd: Box<dyn Command>,
        ctx: &mut CommandContext,
    ) -> Result<()> {
        cmd.execute(ctx)?;
        self.redo_stack.clear();
        self.undo_stack.push(cmd);

        // Enforce max depth -- drain from the front in one shot rather than
        // calling `remove(0)` in a loop (which is O(n) per removal).
        if self.undo_stack.len() > self.max_depth {
            let excess = self.undo_stack.len() - self.max_depth;
            self.undo_stack.drain(..excess);
        }

        Ok(())
    }

    /// Undoes the most recent command, moving it to the redo stack.
    ///
    /// Emits an [`UndoPerformed`] event on success. Returns `Ok(true)` if a
    /// command was undone, `Ok(false)` if the stack was empty.
    pub fn undo(&mut self, ctx: &mut CommandContext) -> Result<bool> {
        if let Some(mut cmd) = self.undo_stack.pop() {
            let desc = cmd.description().to_owned();
            cmd.undo(ctx)?;
            ctx.events.publish(UndoPerformed { description: desc });
            self.redo_stack.push(cmd);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Redoes the most recently undone command, moving it back to the undo stack.
    ///
    /// Emits a [`RedoPerformed`] event on success. Returns `Ok(true)` if a
    /// command was redone, `Ok(false)` if the stack was empty.
    pub fn redo(&mut self, ctx: &mut CommandContext) -> Result<bool> {
        if let Some(mut cmd) = self.redo_stack.pop() {
            let desc = cmd.description().to_owned();
            cmd.execute(ctx)?;
            ctx.events.publish(RedoPerformed { description: desc });
            self.undo_stack.push(cmd);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Returns `true` if there is at least one command to undo.
    #[inline]
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Returns `true` if there is at least one command to redo.
    #[inline]
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Clears both undo and redo stacks.
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}

// ─────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::World;
    use crate::events::EventBus;
    use std::sync::atomic::{AtomicI32, Ordering};
    use std::sync::Arc;

    /// A trivial command that increments/decrements a shared counter.
    struct IncrementCommand {
        amount: i32,
        counter: Arc<AtomicI32>,
    }

    impl Command for IncrementCommand {
        fn execute(&mut self, _ctx: &mut CommandContext) -> Result<()> {
            self.counter.fetch_add(self.amount, Ordering::SeqCst);
            Ok(())
        }

        fn undo(&mut self, _ctx: &mut CommandContext) -> Result<()> {
            self.counter.fetch_sub(self.amount, Ordering::SeqCst);
            Ok(())
        }

        fn description(&self) -> &str {
            "Increment counter"
        }
    }

    fn make_cmd(amount: i32, counter: &Arc<AtomicI32>) -> Box<dyn Command> {
        Box::new(IncrementCommand {
            amount,
            counter: Arc::clone(counter),
        })
    }

    #[test]
    fn execute_and_undo() {
        let counter = Arc::new(AtomicI32::new(0));
        let mut world = World::new();
        let events = EventBus::new();
        let undo_rx = events.subscribe::<UndoPerformed>();

        let mut history = CommandHistory::new();
        let mut ctx = CommandContext::new(&mut world, &events);

        history.execute(make_cmd(5, &counter), &mut ctx).unwrap();
        assert_eq!(counter.load(Ordering::SeqCst), 5);
        assert!(history.can_undo());
        assert!(!history.can_redo());

        history.undo(&mut ctx).unwrap();
        assert_eq!(counter.load(Ordering::SeqCst), 0);
        assert!(!history.can_undo());
        assert!(history.can_redo());

        let received = undo_rx.drain();
        assert_eq!(received.len(), 1);
        assert_eq!(received[0].description, "Increment counter");
    }

    #[test]
    fn redo_after_undo() {
        let counter = Arc::new(AtomicI32::new(0));
        let mut world = World::new();
        let events = EventBus::new();
        let redo_rx = events.subscribe::<RedoPerformed>();

        let mut history = CommandHistory::new();
        let mut ctx = CommandContext::new(&mut world, &events);

        history.execute(make_cmd(3, &counter), &mut ctx).unwrap();
        history.undo(&mut ctx).unwrap();
        history.redo(&mut ctx).unwrap();
        assert_eq!(counter.load(Ordering::SeqCst), 3);

        let received = redo_rx.drain();
        assert_eq!(received.len(), 1);
    }

    #[test]
    fn execute_clears_redo() {
        let counter = Arc::new(AtomicI32::new(0));
        let mut world = World::new();
        let events = EventBus::new();
        let mut history = CommandHistory::new();
        let mut ctx = CommandContext::new(&mut world, &events);

        history.execute(make_cmd(1, &counter), &mut ctx).unwrap();
        history.undo(&mut ctx).unwrap();
        assert!(history.can_redo());

        history.execute(make_cmd(2, &counter), &mut ctx).unwrap();
        assert!(!history.can_redo());
    }

    #[test]
    fn max_depth_enforced() {
        let counter = Arc::new(AtomicI32::new(0));
        let mut world = World::new();
        let events = EventBus::new();
        let mut history = CommandHistory::with_max_depth(3);
        let mut ctx = CommandContext::new(&mut world, &events);

        for i in 0..5 {
            history
                .execute(make_cmd(i + 1, &counter), &mut ctx)
                .unwrap();
        }
        assert_eq!(history.undo_stack.len(), 3);
    }

    #[test]
    fn clear_stacks() {
        let counter = Arc::new(AtomicI32::new(0));
        let mut world = World::new();
        let events = EventBus::new();
        let mut history = CommandHistory::new();
        let mut ctx = CommandContext::new(&mut world, &events);

        history.execute(make_cmd(1, &counter), &mut ctx).unwrap();
        history.clear();
        assert!(!history.can_undo());
        assert!(!history.can_redo());
    }
}
