$fn = 100;

module actual_pcb_board_outline() {
    rotate(a=180) {
        translate([0, 0])
            linear_extrude(height=10)
            import(file = "micropad/micropad-brd.svg", center=true);
    }
}

//actual_pcb_board_outline();

total_1u_count = 2;
row_count = 2;

switch_cutout_1u_width = 14.0;
switch_cutout_1u_length = 14.0;
switch_cutout_1u_padding = 6.50;
switch_cutout_1u_pitch = switch_cutout_1u_width + switch_cutout_1u_padding;

top_plate_padding_top_bottom = 0;
top_plate_padding_left_right = 0;
top_plate_height = 1.5;
top_plate_width = (((row_count * switch_cutout_1u_pitch) + top_plate_padding_top_bottom) * 1.854) + 1.5;
top_plate_length = (((total_1u_count * switch_cutout_1u_pitch) + top_plate_padding_left_right) * 1.03) + 1.0;
top_plate_corner_radius = 3;

echo("Top plate dimensions are w=", top_plate_width, ",l=", top_plate_length);

cherry_top_height = 3.6;
cherry_middle_height = 6.6;
cherry_bottom_width = 15.6;
cherry_bottom_length = 15.6;

dsa_keycap_bottom_width = 18.415;
dsa_keycap_bottom_length = 18.415;
dsa_keycap_top_length = 12.7;
dsa_keycap_top_width = 12.7;
dsa_keycap_height = 7.39;

encoder_width = 13;
encoder_length = 15;

encoder_base_height = 6.75;
encoder_shaft_height = 15.591;
encoder_shaft_diameter = 6.0;

apa102_length = 7;
apa102_width = 5;

usb_width = 9;
usb_length = 10.1;
usb_height = 2.56;

mounting_hole_radius = 1.5 + 0.4;
mounting_hole_head_radius = mounting_hole_radius + 0.8;
mounting_hole_head_height = 1.72;

mounting_hole_x_offset = 65.58 / 2; // Measured from the PCB directly
mounting_hole_y_offset = 36.83 / 2; // Measured form the PCB directly

pcb_width = 76.4;
pcb_length = 45.13;
pcb_height = 1.6;

standoff_radius = 2;
standoff_height = 8;

plate_standoff_radius = 2;
plate_standoff_height = 5;

case_chamfer_size = 2;
case_wall_thickness = 3;
case_length = pcb_length + 1.5;
case_width = pcb_width + 2.2;

case_height = standoff_height + pcb_height + top_plate_height + (case_wall_thickness / 2) + plate_standoff_height - (case_chamfer_size / 2);

echo("Case height is: ", case_height);

standoff_offset_z = (case_height / 2) - (case_wall_thickness / 2);
pcb_offset_z = -standoff_offset_z + standoff_height;
plate_standoff_offset_z = pcb_offset_z + pcb_height;
top_plate_offset_z = plate_standoff_offset_z + plate_standoff_height;

echo("Top plate PCB distance is: ", top_plate_offset_z - pcb_offset_z);

encoder_offset_x = 0;
encoder_offset_y = -(top_plate_width / 2 / 2);
encoder_offset_z = pcb_offset_z + (encoder_base_height / 2) + pcb_height;

knob_radius = 12;
knob_height = 14;

MUTE_PALLETTE = [
    [39, 86, 123],
    [182, 59, 59],
    [249, 196, 20]
];

KEYCAP_PALLETTE = MUTE_PALLETTE;

C1 = [KEYCAP_PALLETTE[0][0] / 255, KEYCAP_PALLETTE[0][1] / 255, KEYCAP_PALLETTE[0][2] / 255];
C2 = [KEYCAP_PALLETTE[1][0] / 255, KEYCAP_PALLETTE[1][1] / 255, KEYCAP_PALLETTE[1][2] / 255];
C3 = [KEYCAP_PALLETTE[2][0] / 255, KEYCAP_PALLETTE[2][1] / 255, KEYCAP_PALLETTE[2][2] / 255];

KEYCAP_COLORS = [
    [C1, C1],
    [C1, C1],
];

union() {
    pcb();
    top_plate(top_plate_height);
    case();
//    plate(top_plate_length, top_plate_width, top_plate_corner_radius);
}

module pcb() {
    color("green") {
        translate([0, 0, pcb_offset_z]) {
            linear_extrude(height=pcb_height) {
                difference() {
                    plate(top_plate_length, top_plate_width, 0);
                    mounting_holes();
                }
            }
        }
    }
}

module top_plate(height=0) {
    translate([0, 0, top_plate_offset_z]) {
//        linear_extrude(height=height) {
            difference() {
                plate(top_plate_length, top_plate_width);
                union() {
                    translate([0.75, -2.5]) {
                        row_0_switch_cutout();
                        row_1_switch_cutout();
                        encoder_cutout();
                        apa102_cutout();
                    }
                    mounting_holes();
                }
            }
//        }
    }
}

module case() {
    module main_cutout() {
        translate([0, 0, case_wall_thickness]) {
            hull() {
                cube([case_length - case_wall_thickness,
                      case_width - case_wall_thickness,
                      case_height - case_wall_thickness], center=true);
                3d_rounded_corners(length=case_length - case_wall_thickness - (case_chamfer_size / 2),
                                   width=case_width - case_wall_thickness - (case_chamfer_size / 2),
                                   height=case_height - case_wall_thickness, corner_radius=2);
            }
        }
    }

    module usb_case_cutout() {
        height = usb_height + 1;
        translate([0, -(top_plate_width / 2) - 5, pcb_offset_z + (height / 2) + pcb_height]) {
            hull() {
                cube([usb_length, usb_width, height], center=true);
                3d_rounded_corners(usb_length, usb_width, height, corner_radius=1);
            }
        }
    }

    %difference() {
        //color("cyan")
            hull() {
                cube([case_length, case_width, case_height], center=true);
                3d_rounded_corners(length=case_length, width=case_width, height=case_height - case_chamfer_size, corner_radius=2);
            }
        union() {
            main_cutout();
            usb_case_cutout();
            translate([0, 0, -(case_height / 2) - 0.1]) {
                linear_extrude(height = mounting_hole_head_height) {
                    mounting_holes(radius = mounting_hole_head_radius);
                }
            }
            translate([0, 0, -(case_height / 2) - (standoff_height / 3)]) {
                linear_extrude(height = standoff_height * 2) {
                    mounting_holes(radius = mounting_hole_radius);
                }
            }
        }
    }
    standoffs();
    encoder();
}

module encoder() {
    top_of_encoder_base = encoder_offset_z + (encoder_base_height / 2);
    encoder_shaft_height_offset = top_of_encoder_base + (encoder_shaft_height / 2);

    module encoder_base() {
        translate([0, 0, encoder_offset_z]) {
            color("red")
                rotate(90)
                cube([encoder_width,
                      encoder_length,
                      encoder_base_height],
                     center=true);
        }
    }

    module encoder_shaft() {
        translate([0, 0, encoder_shaft_height_offset]) {
            color("blue")
                cylinder(h=encoder_shaft_height,
                         d=encoder_shaft_diameter,
                         center=true);
        }
    }

    module knob() {
        translate([0, 0, encoder_shaft_height_offset - (knob_height / 3)]) {
            color("silver")
                cylinder(r=knob_radius, h=knob_height);
        }
    }

    translate([0, encoder_offset_y]) {
        encoder_base();
        encoder_shaft();
        knob();
    }
}

module encoder_cutout() {
    translate([0, encoder_offset_y])
        hull() {
        square([encoder_length+2, encoder_width], center=true);
        rounded_corners(encoder_length, encoder_width, 1);
    }
}

module apa102_cutout() {
    translate([0, -(top_plate_width / 2 / 2 / 2)])
        hull() {
        square([apa102_length, apa102_width], center=true);
        rounded_corners(apa102_length, apa102_width, 1);
    }
}

module standoffs() {
    // Lower standoff
    translate([0, 0, -standoff_offset_z]) {
        translate([-mounting_hole_y_offset, -mounting_hole_x_offset])
            cylinder(r=standoff_radius, h=standoff_height);
        translate([mounting_hole_y_offset, -mounting_hole_x_offset])
            cylinder(r=standoff_radius, h=standoff_height);
        translate([mounting_hole_y_offset, mounting_hole_x_offset])
            cylinder(r=standoff_radius, h=standoff_height);
        translate([-mounting_hole_y_offset, mounting_hole_x_offset])
            cylinder(r=standoff_radius, h=standoff_height);
    }

    // Middle standoff
    translate([0, 0, plate_standoff_offset_z]) {
        translate([-mounting_hole_y_offset, -mounting_hole_x_offset])
            cylinder(r=plate_standoff_radius, h=plate_standoff_height);
        translate([mounting_hole_y_offset, -mounting_hole_x_offset])
            cylinder(r=plate_standoff_radius, h=plate_standoff_height);
        translate([mounting_hole_y_offset, mounting_hole_x_offset])
            cylinder(r=plate_standoff_radius, h=plate_standoff_height);
        translate([-mounting_hole_y_offset, mounting_hole_x_offset])
            cylinder(r=plate_standoff_radius, h=plate_standoff_height);
    }
}

module mounting_holes(radius = mounting_hole_radius) {
    translate([-mounting_hole_y_offset, -mounting_hole_x_offset])
        circle(r=radius);
    translate([mounting_hole_y_offset, -mounting_hole_x_offset])
        circle(r=radius);
    translate([mounting_hole_y_offset, mounting_hole_x_offset])
        circle(r=radius);
    translate([-mounting_hole_y_offset, mounting_hole_x_offset])
        circle(r=radius);
}

module row_0_switch_cutout() {
    row_switch_cutout(row=0, switch_offset=0.5, switch_size=2, cutout_count=1, height=top_plate_height);
}

module row_1_switch_cutout() {
    row_switch_cutout(row=1, switch_offset=0, cutout_count=total_1u_count, height=top_plate_height);
}

module row_switch_cutout(row, switch_offset, cutout_count, switch_size=1, height=0, add_small_stabilizer=false) {
    start_x_offset = -(top_plate_length / 2) + (switch_cutout_1u_pitch / 2) + (top_plate_padding_left_right / 2) + (switch_cutout_1u_pitch * switch_offset);
    start_y_offset = (top_plate_width / 2) - (switch_cutout_1u_pitch * (row + 1)) + (switch_cutout_1u_pitch / 2) - (top_plate_padding_top_bottom / 2);

    for (i = [0:cutout_count - 1]) {
        x_offset = start_x_offset + (i * switch_cutout_1u_pitch);
        y_offset = start_y_offset;

        cherry_mx_cutout(x_offset,
                         y_offset,
                         switch_cutout_1u_width,
                         switch_cutout_1u_length,
                         add_small_stabilizer);
        translate([0, 0, height]) {
            //%dsa_keycap(x_offset, y_offset, KEYCAP_COLORS[row][i + floor(switch_offset)], switch_size);
            //%cherry_mx_switch(x_offset, y_offset);
        }
    }
}

module 3d_rounded_corners(length, width, height, corner_radius) {
    translate([-(length / 2), (width / 2)])
        cylinder(h=height, r=corner_radius, center=true);
    translate([-(length / 2), -(width / 2)])
        cylinder(h=height, r=corner_radius, center=true);
    translate([(length / 2), (width / 2)])
        cylinder(h=height, r=corner_radius, center=true);
    translate([(length / 2), -(width / 2)])
        cylinder(h=height, r=corner_radius, center=true);
}

module rounded_corners(length, width, corner_radius) {
    translate([-(length / 2), (width / 2)])
        circle(r=corner_radius);
    translate([-(length / 2), -(width / 2)])
        circle(r=corner_radius);
    translate([(length / 2), (width / 2)])
        circle(r=corner_radius);
    translate([(length / 2), -(width / 2)])
        circle(r=corner_radius);
}

module plate(length, width, corner_radius) {
    color("gray", 1.0)
        hull() {
        square([length,
                width],
               center = true);
        rounded_corners(length, width, corner_radius);
    }
}

module cherry_mx_cutout(x, y, switch_cutout_width, switch_cutout_length, add_small_stabilizer=false) {
    module rounded_corners() {
        corner_radius = 0.3;
        translate([x + (switch_cutout_length / 2), y + (switch_cutout_width / 2)])
            circle(r = corner_radius);
        translate([x + (switch_cutout_length / 2), y - (switch_cutout_width / 2)])
            circle(r = corner_radius);
        translate([x - (switch_cutout_length / 2), y + (switch_cutout_width / 2)])
            circle(r = corner_radius);
        translate([x - (switch_cutout_length / 2), y - (switch_cutout_width / 2)])
            circle(r = corner_radius);
    }

    hull() {
        translate([x, y])
            square([switch_cutout_length,
                    switch_cutout_width],
                center=true);
        rounded_corners();
    }

    if (add_small_stabilizer) {
        small_stabilizer(x - switch_cutout_length, y, false);
        small_stabilizer(x + switch_cutout_length, y, true);
    }
}

module cherry_mx_switch(x, y) {
    offset_z = cherry_middle_height / 2;
    translate([x, y, offset_z]) {
        color("black", 1.0)
        linear_extrude(height = cherry_middle_height,
                       center = true,
                       scale = 0.69)
            square([cherry_bottom_width,
                    cherry_bottom_length],
                   center=true);
    }
}

module dsa_keycap(x, y, cap_color, switch_size) {
    offset_z = (dsa_keycap_height / 2) + cherry_middle_height - cherry_top_height;
    translate([x, y, offset_z]) {
        color(cap_color, 1.0)
            linear_extrude(height = dsa_keycap_height, center = true, scale = 0.69)
            square([dsa_keycap_bottom_width * switch_size,
                    dsa_keycap_bottom_length],
                   center=true);
    }
}


module small_stabilizer(x, y, right=false) {
    stabilizer_length = 6.65 + 0.1;
    stabilizer_width = 13.46 + 0.15;

    bottom_square_length = 3.04 + 0.1;
    bottom_square_width = 1.16 + 0.254;

    left_square_length = 0.762;
    left_square_width = 2.79 + 0.2;
    left_square_width_offset = 0.5;

    right_square_length = switch_cutout_1u_length - (stabilizer_length);
    right_square_width = stabilizer_width - (2 * (0.81 + 0.1));

    translate([x, y]) {
        mirror([right ? 1 : 0, 0, 0]) {
            union() {
                square([stabilizer_length,
                        stabilizer_width],
                       center=true);
                translate([0, -(stabilizer_width / 2) - (bottom_square_width / 2) + 0.01]) {
                    square([bottom_square_length,
                            bottom_square_width],
                           center=true);
                }
                translate([-(stabilizer_length / 2) - (left_square_length / 2) + 0.01, (left_square_width / 2) - left_square_width_offset]) {
                    square([left_square_length,
                            left_square_width],
                           center=true);
                }
                translate([(stabilizer_length / 2) + (right_square_length / 2) - 0.01, 0]) {
                    square([right_square_length,
                            right_square_width],
                           center=true);
                }
            }
        }
    }
}

