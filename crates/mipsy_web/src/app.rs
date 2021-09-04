use yew::{prelude::*, services::{ConsoleService, ReaderService, reader::FileData}, web_sys::File};
use crate::components::{pagebackground::PageBackground, navbar::NavBar};

pub enum Msg {
    FileChanged(File),
    FileRead(FileData),
}

pub struct App {
    // `ComponentLink` is like a reference to a component.
    // It can be used to send messages to the component
    link: ComponentLink<Self>,
    file: Option<String>,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { link, file: None }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {

        match msg {
            Msg::FileChanged(file) => {
                ConsoleService::info("file changed msg");
                // FIXME -- check result
                let result = ReaderService::read_file(file, self.link.callback(|file_data| Msg::FileRead(file_data))); 
                
                if let Err(err) = result {
                    ConsoleService::error(&format!("{:?}",err));
                }
                false
            }
            Msg::FileRead(file_data) => {
                ConsoleService::info(&format!("{:?}", file_data));
                // TODO -- this should not be lossy
                self.file = Some(String::from_utf8_lossy(&file_data.content).to_string()); 
                true
            }
        }
        
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        // Should only return "true" if new properties are different to
        // previously received properties.
        // This component has no properties so we will always return "false".
        false
    }
    
    fn view(&self) -> Html {
        let onchange = self.link.batch_callback(|event| {
            ConsoleService::info("onchange fired");
            match event {
                ChangeData::Files(file_list) => {
                    if let Some(file) = file_list.item(0) {
                        ConsoleService::info(&format!("{:?}", file.name() ));
                        Some(Msg::FileChanged(file))
                    } else {
                        None
                    }
                },
                _ => None, 

            } 
        } );

        ConsoleService::info(&format!("{:?}", self.file));

        html! {
            <PageBackground>
                <NavBar onchange=onchange/>
                <div>
                {
                    if let Some(data) = self.file.as_deref() {
                        data
                    } else {
                        ""
                    }
                }
                </div>
            </PageBackground>
        }
    }
}
