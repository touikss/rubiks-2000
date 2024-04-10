#![allow(dead_code)] // This macros disable all the "unsed function/variable ..." warning, to simplify clarity during dev

mod cube;
mod graphic_cube;
use std::sync::{Arc, Mutex};

use cube::cube::Cube;
use graphic_cube::{
    gcube::{Gcube, Move::*},
    gtk_binding::update_gtk_image,
};

extern crate kiss3d;
extern crate nalgebra as na;

use gtk::prelude::*;
use kiss3d::{camera::{Camera, FirstPerson}, light::Light, ncollide3d::math::Point, window::Window};

fn e() {
    // Initialiser GTK.
    gtk::init().expect("Failed to initialize GTK.");

    // Créer une nouvelle fenêtre GTK.
    let gtk_window = gtk::Window::new(gtk::WindowType::Toplevel);
    gtk_window.set_title("Exemple GTK");
    gtk_window.set_default_size(350, 250);

    // Créer une image GTK.
    let mut image = gtk::Image::new();
    gtk_window.add(&image);

    // Afficher tous les widgets GTK.
    gtk_window.show_all();

    // Créer la fenêtre kiss3D.
    let mut window = Window::new_hidden("Rubik's 2000");
    window.set_light(Light::StickToCamera);
    
    let mut camera = FirstPerson::new(Point::new(-10., 10., 6.), Point::new(0., 0., 0.));


    let cube = Cube::new_filled(3);
    let mut gcube = Gcube::new(cube, &mut window);

    // Variable partagée pour vérifier si la touche "espace" est pressée
    let space_pressed = Arc::new(Mutex::new(false));

    // Clôtures pour gérer les événements de pression et de libération de la touche "espace"
    let space_pressed_key_press = Arc::clone(&space_pressed);
    let space_pressed_key_release = Arc::clone(&space_pressed);

    gtk_window.connect_key_press_event(move |_, key_event| {
        if key_event.keyval() == gtk::gdk::keys::constants::space {
            *space_pressed_key_press.lock().unwrap() = true;
        }
        gtk::glib::Propagation::Stop
    });

    gtk_window.connect_key_release_event(move |_, key_event| {
        if key_event.keyval() == gtk::gdk::keys::constants::space {
            *space_pressed_key_release.lock().unwrap() = false;
        }
        gtk::glib::Propagation::Stop
    });

    // Boucle principale
    loop {
        update_gtk_image(&mut window, &mut image);

        // Rendre la fenêtre kiss3D
        if !window.render_with_camera(&mut camera) {
            break;
        }

        // Vérifier si la touche "espace" est pressée
        if *space_pressed.lock().unwrap() {
            println!("La touche espace est pressée !");
            gcube.apply_move(&mut window, H, 0, &mut image, &mut camera);
        }
    }
}


