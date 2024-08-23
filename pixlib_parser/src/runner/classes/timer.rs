use std::{any::Any, cell::RefCell};

use super::super::content::EventHandler;
use super::super::initable::Initable;
use super::super::parsers::{discard_if_empty, parse_bool, parse_event_handler, parse_i32};

use crate::{common::DroppableRefMut, parser::ast::ParsedScript, runner::InternalEvent};

use super::super::common::*;
use super::super::*;
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
    pub on_tick: HashMap<String, Arc<ParsedScript>>, // ONTICK signal
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
    pub on_tick: HashMap<String, Arc<ParsedScript>>, // ONTICK signal
}

impl EventHandler for TimerEventHandlers {
    fn get(&self, name: &str, argument: Option<&str>) -> Option<&Arc<ParsedScript>> {
        match name {
            "ONDONE" => self.on_done.as_ref(),
            "ONINIT" => self.on_init.as_ref(),
            "ONSIGNAL" => self.on_signal.as_ref(),
            "ONTICK" => argument
                .and_then(|a| self.on_tick.get(a))
                .or(self.on_tick.get("")),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Timer {
    parent: Arc<CnvObject>,

    state: RefCell<TimerState>,
    event_handlers: TimerEventHandlers,

    max_ticks: Option<usize>,
}

impl Timer {
    pub fn from_initial_properties(parent: Arc<CnvObject>, props: TimerProperties) -> Self {
        let interval_ms = props.elapse.unwrap_or_default() as usize;
        let is_enabled = props.enabled.unwrap_or(true);
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
            max_ticks: match props.ticks.unwrap_or_default() {
                0 => None,
                i => Some(i as usize),
            },
        }
    }

    // custom

    pub fn step(&self, seconds: f64) -> RunnerResult<()> {
        let context = RunnerContext::new_minimal(&self.parent.parent.runner, &self.parent);
        self.state.borrow_mut().step(context, seconds * 1000f64)
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
                .set(arguments[0].to_int() as f64)
                .map(|_| None),
            CallableIdentifier::Method("SETELAPSE") => self
                .state
                .borrow_mut()
                .set_elapse(arguments[0].to_int() as usize)
                .map(|_| None),
            CallableIdentifier::Event(event_name) => {
                if let Some(code) = self
                    .event_handlers
                    .get(event_name, arguments.first().map(|v| v.to_str()).as_deref())
                {
                    code.run(context)?;
                }
                Ok(None)
            }
            ident => Err(RunnerError::InvalidCallable {
                object_name: self.parent.name.clone(),
                callable: ident.to_owned(),
            }),
        }
    }

    fn new_content(
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
            .map(parse_event_handler)
            .transpose()?;
        let on_init = properties
            .remove("ONINIT")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let on_signal = properties
            .remove("ONSIGNAL")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let mut on_tick = HashMap::new();
        for (k, v) in properties.iter() {
            if k == "ONTICK" {
                on_tick.insert(String::from(""), parse_event_handler(v.to_owned())?);
            } else if let Some(argument) = k.strip_prefix("ONTICK^") {
                on_tick.insert(String::from(argument), parse_event_handler(v.to_owned())?);
            }
        }
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
    fn initialize(&self, context: RunnerContext) -> RunnerResult<()> {
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

    // custom

    pub fn step(&mut self, context: RunnerContext, duration_ms: f64) -> RunnerResult<()> {
        // eprintln!("Stepping timer {} by {} ms", timer.parent.name, duration_ms);
        let CnvContent::Timer(timer) = &context.current_object.content else {
            panic!();
        };
        if !self.is_enabled
            || self.is_paused
            || self.interval_ms == 0
            || timer
                .max_ticks
                .map(|max| self.current_ticks >= max)
                .unwrap_or_default()
        {
            return Ok(());
        }
        self.current_ms -= duration_ms;
        while self.current_ms < 0.0
            && timer
                .max_ticks
                .map(|max| self.current_ticks < max)
                .unwrap_or(true)
        {
            self.current_ms += self.interval_ms as f64;
            self.current_ticks += 1;
            context
                .runner
                .internal_events
                .borrow_mut()
                .use_and_drop_mut(|events| {
                    events.push_back(InternalEvent {
                        object: context.current_object.clone(),
                        callable: CallableIdentifier::Event("ONTICK").to_owned(),
                        arguments: vec![CnvValue::Integer(self.current_ticks as i32)],
                    })
                });
        }
        if timer
            .max_ticks
            .map(|max| self.current_ticks >= max)
            .unwrap_or_default()
        {
            self.current_ms = 0.0;
            self.is_paused = true;
        }
        Ok(())
    }
}
