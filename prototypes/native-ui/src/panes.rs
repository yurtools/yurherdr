//! One transfer contract for every pane. Sessions are owned by the workspace,
//! never by a view or a window-local focus handle.
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PaneId {
    Composer,
    Document,
    Terminal,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowId(pub u64);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewLease {
    pub pane: PaneId,
    pub window: WindowId,
    epoch: u64,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Transfer {
    source: ViewLease,
    destination: WindowId,
    epoch: u64,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveError {
    Hidden,
    Busy,
    Stale,
    SameWindow,
    Exhausted,
}

#[derive(Default)]
struct Placement {
    view: Option<ViewLease>,
    pending: Option<Transfer>,
    epoch: u64,
}

pub struct Panes {
    placements: BTreeMap<PaneId, Placement>,
}
impl Default for Panes {
    fn default() -> Self {
        Self {
            placements: [PaneId::Composer, PaneId::Document, PaneId::Terminal]
                .into_iter()
                .map(|id| (id, Placement::default()))
                .collect(),
        }
    }
}
impl Panes {
    pub fn view(&self, pane: PaneId) -> Option<ViewLease> {
        self.placements[&pane].view
    }

    pub fn reopen(&mut self, pane: PaneId, window: WindowId) -> Result<ViewLease, MoveError> {
        let p = self
            .placements
            .get_mut(&pane)
            .expect("fixed pane inventory");
        if p.view.is_some() || p.pending.is_some() {
            return Err(MoveError::Busy);
        }
        p.epoch = p.epoch.checked_add(1).ok_or(MoveError::Exhausted)?;
        let view = ViewLease {
            pane,
            window,
            epoch: p.epoch,
        };
        p.view = Some(view);
        Ok(view)
    }

    /// Reserve destination before creating its native window. Source remains
    /// authoritative until the destination explicitly confirms readiness.
    pub fn reserve(
        &mut self,
        source: ViewLease,
        destination: WindowId,
    ) -> Result<Transfer, MoveError> {
        let p = self
            .placements
            .get_mut(&source.pane)
            .expect("fixed pane inventory");
        if p.view != Some(source) {
            return Err(MoveError::Stale);
        }
        if p.pending.is_some() {
            return Err(MoveError::Busy);
        }
        if source.window == destination {
            return Err(MoveError::SameWindow);
        }
        let epoch = p.epoch.checked_add(1).ok_or(MoveError::Exhausted)?;
        p.epoch = epoch;
        let transfer = Transfer {
            source,
            destination,
            epoch,
        };
        p.pending = Some(transfer);
        Ok(transfer)
    }

    /// Call only after native destination creation/readiness succeeds. This
    /// atomic switch invalidates source resize/input rights exactly once.
    pub fn ready(&mut self, transfer: Transfer) -> Result<ViewLease, MoveError> {
        let p = self
            .placements
            .get_mut(&transfer.source.pane)
            .expect("fixed pane inventory");
        if p.pending != Some(transfer) || p.view != Some(transfer.source) {
            return Err(MoveError::Stale);
        }
        let view = ViewLease {
            pane: transfer.source.pane,
            window: transfer.destination,
            epoch: transfer.epoch,
        };
        p.view = Some(view);
        p.pending = None;
        Ok(view)
    }

    pub fn rollback(&mut self, transfer: Transfer) -> Result<(), MoveError> {
        let p = self
            .placements
            .get_mut(&transfer.source.pane)
            .expect("fixed pane inventory");
        if p.pending != Some(transfer) {
            return Err(MoveError::Stale);
        }
        p.pending = None;
        Ok(())
    }

    pub fn close(&mut self, view: ViewLease) -> Result<(), MoveError> {
        let p = self
            .placements
            .get_mut(&view.pane)
            .expect("fixed pane inventory");
        if p.view != Some(view) {
            return Err(MoveError::Stale);
        }
        p.view = None;
        p.pending = None;
        Ok(())
    }
    pub fn accepts(&self, view: ViewLease) -> bool {
        self.view(view.pane) == Some(view)
    }
}
