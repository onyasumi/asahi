use std::error::Error;
use zbus::export::futures_util::StreamExt;
use crate::location::geoclue::{AccuracyLevel, LocationProxy, ManagerProxy};


pub struct LocationProvider<'a> {
    
    on_coords_acquired: &'a mut dyn FnMut(f64, f64)
    
}

impl<'a> LocationProvider<'a> {
    
    pub fn new(on_coords_acquired: &'a mut dyn FnMut(f64, f64)) -> Self {
        Self {
            on_coords_acquired
        }
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn Error>> {

        let conn = zbus::Connection::system().await?;

        let gclue_manager = ManagerProxy::new(&conn).await?;
        let gclue_client = gclue_manager.get_client().await?;
        gclue_client.set_desktop_id("asahi").await?;
        gclue_client.set_distance_threshold(10000).await?;      // 10 km threshold
        gclue_client.set_requested_accuracy_level(AccuracyLevel::City as u32).await?;

        let mut location_updated = gclue_client.receive_location_updated().await?;

        gclue_client.start().await?;
        
        println!("Location Provider Started");

        loop {
            
            let signal = location_updated.next().await.unwrap();

            let args = signal.args()?;

            let location = LocationProxy::builder(&conn)
                .path(args.new())?
                .build().await?;

            let latitude = location.latitude().await?;
            let longitude = location.longitude().await?;
            
            println!("Latitude: {latitude}, Longitude: {longitude}");

            (self.on_coords_acquired)(latitude, longitude);

        }
        
    }

}