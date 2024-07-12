use std::io;

use reedline::{DefaultPrompt, DefaultPromptSegment, Reedline, Signal};

use super::ShellTokens;

pub enum RunState {
    Success,
    CtrlD,
    CtrlC,
}

pub struct Shell {
    editor: Reedline,
    normal_prompt: DefaultPrompt,
    pending_prompt: DefaultPrompt,
    tokens: ShellTokens,
}

impl Default for Shell {
    fn default() -> Self {
        Self {
            editor: Reedline::create(),
            normal_prompt: DefaultPrompt::new(
                DefaultPromptSegment::Basic(format!("boba ")),
                DefaultPromptSegment::Empty,
            ),
            pending_prompt: DefaultPrompt::new(
                DefaultPromptSegment::Basic(format!("  ...")),
                DefaultPromptSegment::Empty,
            ),
            tokens: ShellTokens::new(),
        }
    }
}

impl Shell {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn read_statement(&mut self) -> io::Result<RunState> {
        loop {
            // choose a prompt
            let prompt = match self.tokens.is_ready() {
                false => &self.pending_prompt,
                true => &self.normal_prompt,
            };

            // get the text
            let text = match self.editor.read_line(prompt)? {
                Signal::Success(text) => text,
                Signal::CtrlC => {
                    return Ok(RunState::CtrlC);
                }
                Signal::CtrlD => {
                    return Ok(RunState::CtrlD);
                }
            };

            // load the tokens
            if !self.tokens.load(text) {
                self.tokens.close_all_blocks();
                continue;
            }

            // check if tokens are ready
            if !self.tokens.is_ready() {
                continue;
            }
        }
    }
}
