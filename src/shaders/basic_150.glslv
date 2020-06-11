#version 150

in vec3 a_Pos;
in vec4 a_Color;

uniform mat4 u_Model;
uniform mat4 u_View;

out vec4 v_Color;

void main() {
    v_Color = a_Color;
    gl_Position = u_View * u_Model * vec4(a_Pos, 1.0);
}
