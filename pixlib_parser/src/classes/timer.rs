use std::{any::Any, cell::RefCell};

use initable::Initable;
use parsers::{discard_if_empty, parse_bool, parse_i32, parse_program};

use crate::{ast::ParsedScript, common::DroppableRefMut, runner::InternalEvent};

use super::*;

#[derive(Debug, Clone)]
pub struct TimerProperties {
    // TIMER
    pub elapse: Option<i32>,   // ELAPSE
    pub enabled: Option<bool>, // ENABLED
    pub ticks: Option<i32>,    // TICKS

    pub on_done: Option<Arc<ParsedScript>>,   // ONDONE signal
    pub on_init: Option<Arc<ParsedScript>>,   // ONINIT signal
    pub on_signal: Option<Arc<ParsedScript>>, // ONSIGNAL signal
    pub on_tick: Option<Arc<ParsedScript>>,   // ONTICK signal
}

#[derive(Debug, Clone, Default)]
struct TimerState {
    // initialized from properties
    pub interval_ms: usize,
    pub is_enabled: bool,

    // general timer-related
    pub is_paused: bool,
    pub current_ms: f64,
    pub current_ticks: usize,
}

#[derive(Debug, Clone)]
pub struct TimerEventHandlers {
    pub on_done: Option<Arc<ParsedScript>>,   // ONDONE signal
    pub on_init: Option<Arc<ParsedScript>>,   // ONINIT signal
    pub on_signal: Option<Arc<ParsedScript>>, // ONSIGNAL signal
    pub on_tick: Option<Arc<ParsedScript>>,   // ONTICK signal
}

#[derive(Debug, Clone)]
pub struct Timer {
    parent: Arc<CnvObject>,

    state: RefCell<TimerState>,
    event_handlers: TimerEventHandlers,

    max_ticks: usize,
}

impl Timer {
    pub fn from_initial_properties(parent: Arc<CnvObject>, props: TimerProperties) -> Self {
        let interval_ms = props.elapse.unwrap_or_default() as usize;
        let is_enabled = props.enabled.unwrap_or_default();
        Self {
            parent,
            state: RefCell::new(TimerState {
                interval_ms,
                is_enabled,
                current_ms: if is_enabled { interval_ms as f64 } else { 0.0 },
                ..Default::default()
            }),
            event_handlers: TimerEventHandlers {
                on_done: props.on_done,
                on_init: props.on_init,
                on_signal: props.on_signal,
                on_tick: props.on_tick,
            },
            max_ticks: props.ticks.unwrap_or_default() as usize,
        }
    }

    ///

    pub fn step(&self, seconds: f64) -> RunnerResult<()> {
        self.state.borrow_mut().step(&self, seconds * 1000f64)
    }
}

impl CnvType for Timer {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "TIMER"
    }

    fn call_method(
        &self,
        name: CallableIdentifier,
        arguments: &[CnvValue],
        context: RunnerContext,
    ) -> RunnerResult<Option<CnvValue>> {
        // println!("Calling method: {:?} of object: {:?}", name, self);
        match name {
            CallableIdentifier::Method("DISABLE") => {
                self.state.borrow_mut().disable().map(|_| None)
            }
            CallableIdentifier::Method("ENABLE") => self.state.borrow_mut().enable().map(|_| None),
            CallableIdentifier::Method("GETTICKS") => self
                .state
                .borrow()
                .get_ticks()
                .map(|v| Some(CnvValue::Integer(v as i32))),
            CallableIdentifier::Method("PAUSE") => self.state.borrow_mut().pause().map(|_| None),
            CallableIdentifier::Method("RESET") => self.state.borrow_mut().reset().map(|_| None),
            CallableIdentifier::Method("RESUME") => self.state.borrow_mut().resume().map(|_| None),
            CallableIdentifier::Method("SET") => self
                .state
                .borrow_mut()
                .set(arguments[0].to_integer() as f64)
                .map(|_| None),
            CallableIdentifier::Method("SETELAPSE") => self
                .state
                .borrow_mut()
                .set_elapse(arguments[0].to_integer() as usize)
                .map(|_| None),
            CallableIdentifier::Event("ONINIT") => {
                if let Some(v) = self.event_handlers.on_init.as_ref() {
                    v.run(context).map(|_| None)
                } else {
                    Ok(None)
                }
            }
            ident => todo!("{:?} {:?}", self.get_type_id(), ident),
        }
    }

    fn new(
        parent: Arc<CnvObject>,
        mut properties: HashMap<String, String>,
    ) -> Result<CnvContent, TypeParsingError> {
        let elapse = properties
            .remove("ELAPSE")
            .and_then(discard_if_empty)
            .map(parse_i32)
            .transpose()?;
        let enabled = properties
            .remove("ENABLED")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let ticks = properties
            .remove("TICKS")
            .and_then(discard_if_empty)
            .map(parse_i32)
            .transpose()?;
        let on_done = properties
            .remove("ONDONE")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        let on_init = properties
            .remove("ONINIT")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        let on_signal = properties
            .remove("ONSIGNAL")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        let on_tick = properties
            .remove("ONTICK")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        Ok(CnvContent::Timer(Self::from_initial_properties(
            parent,
            TimerProperties {
                elapse,
                enabled,
                ticks,
                on_done,
                on_init,
                on_signal,
                on_tick,
            },
        )))
    }
}

impl Initable for Timer {
    fn initialize(&mut self, context: RunnerContext) -> RunnerResult<()> {
        context
            .runner
            .internal_events
            .borrow_mut()
            .use_and_drop_mut(|events| {
                events.push_back(InternalEvent {
                    object: context.current_object.clone(),
                    callable: CallableIdentifier::Event("ONINIT").to_owned(),
                    arguments: Vec::new(),
                })
            });
        Ok(())
    }
}

impl TimerState {
    pub fn disable(&mut self) -> RunnerResult<()> {
        // DISABLE
        self.is_enabled = false;
        self.current_ms = 0.0;
        self.current_ticks = 0;
        Ok(())
    }

    pub fn enable(&mut self) -> RunnerResult<()> {
        // ENABLE
        self.current_ms = self.interval_ms as f64;
        self.is_enabled = true;
        Ok(())
    }

    pub fn get_ticks(&self) -> RunnerResult<usize> {
        // GETTICKS
        Ok(self.current_ticks)
    }

    pub fn pause(&mut self) -> RunnerResult<()> {
        // PAUSE
        self.is_paused = true;
        Ok(())
    }

    pub fn reset(&mut self) -> RunnerResult<()> {
        // RESET
        self.current_ms = self.interval_ms as f64;
        self.current_ticks = 0;
        Ok(())
    }

    pub fn resume(&mut self) -> RunnerResult<()> {
        // RESUME
        self.is_paused = false;
        Ok(())
    }

    pub fn set(&mut self, current_ms: f64) -> RunnerResult<()> {
        // SET
        self.current_ms = current_ms;
        Ok(())
    }

    pub fn set_elapse(&mut self, interval_ms: usize) -> RunnerResult<()> {
        // SETELAPSE
        self.interval_ms = interval_ms;
        Ok(())
    }

    ///

    pub fn step(&mut self, timer: &Timer, duration_ms: f64) -> RunnerResult<()> {
        if !self.is_enabled
            || self.is_paused
            || self.interval_ms == 0
            || self.current_ticks >= timer.max_ticks
        {
            return Ok(());
        }
        self.current_ms -= duration_ms;
        while self.current_ms < 0.0 && self.current_ticks < timer.max_ticks {
            self.current_ms += self.interval_ms as f64;
            self.current_ticks += 1;
            timer
                .parent
                .parent
                .runner
                .internal_events
                .borrow_mut()
                .use_and_drop_mut(|events| {
                    events.push_back(InternalEvent {
                        object: timer.parent.clone(),
                        callable: CallableIdentifier::Event("ONTICK").to_owned(),
                        arguments: Vec::new(),
                    })
                });
        }
        if self.current_ticks >= timer.max_ticks {
            self.current_ms = 0.0;
            self.is_paused = true;
        }
        Ok(())
    }
}
