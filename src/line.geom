#version 150

uniform uint n;
uniform vec2 window;
uniform float thickness;
uniform float thinning;

layout(lines_adjacency) in;
layout(triangle_strip, max_vertices = 5 ) out;

out float dist;

// heavily based on paul houx's miter polylines
// https://github.com/paulhoux/Cinder-Samples/blob/master/GeometryShader/assets/shaders/lines2.geom
void main() {
    // get the four vertices passed to the shader:
    vec2 p0 = gl_in[0].gl_Position.xy * window;
    vec2 p1 = gl_in[1].gl_Position.xy * window;
    vec2 p2 = gl_in[2].gl_Position.xy * window;
    vec2 p3 = gl_in[3].gl_Position.xy * window;

    // determine the direction of each of the 3 segments (previous, current, next)
    vec2 v0 = normalize( p1 - p0 );
    vec2 v1 = normalize( p2 - p1 );
    vec2 v2 = normalize( p3 - p2 );

    // determine the normal of each of the 3 segments (previous, current, next)
    vec2 n0 = vec2( -v0.y, v0.x );
    vec2 n1 = vec2( -v1.y, v1.x );
    vec2 n2 = vec2( -v2.y, v2.x );

    // determine miter lines by averaging the normals of the 2 segments
    vec2 miter_a = normalize( n0 + n1 );    // miter at start of current segment
    vec2 miter_b = normalize( n1 + n2 );    // miter at end of current segment

    float dist_a = distance(gl_in[0].gl_Position.xy, gl_in[1].gl_Position.xy);
    float dist_b = distance(gl_in[1].gl_Position.xy, gl_in[2].gl_Position.xy);

    float thickness_adjusted = thickness * mix(1.0, 4.0, thinning);
    float thickness_a = min(thickness, thickness_adjusted / mix(1.0, (n * dist_a), thinning));
    float thickness_b = min(thickness, thickness_adjusted / mix(1.0, (n * dist_b), thinning));

    // determine the length of the miter by projecting it onto normal and then inverse it
    float length_a = thickness_a / dot( miter_a, n1 );
    float length_b = thickness_b / dot( miter_b, n1 );

    dist = dist_a;
    if( dot( v0, n1 ) > 0 ) {
        // start at negative miter
        gl_Position = vec4( ( p1 - length_a * miter_a ) / window, 0.0, 1.0 );
        EmitVertex();

        // proceed to positive normal
        gl_Position = vec4( ( p1 + thickness_a * n1 ) / window, 0.0, 1.0 );
        EmitVertex();
    } else {
        // start at negative normal
        gl_Position = vec4( ( p1 - thickness_a * n1 ) / window, 0.0, 1.0 );
        EmitVertex();

        // proceed to positive miter
        gl_Position = vec4( ( p1 + length_a * miter_a ) / window, 0.0, 1.0 );
        EmitVertex();
    }

    dist = dist_b;
    if( dot( v2, n1 ) < 0 ) {
        // proceed to negative miter
        gl_Position = vec4( ( p2 - length_b * miter_b ) / window, 0.0, 1.0 );
        EmitVertex();

        // proceed to positive normal
        gl_Position = vec4( ( p2 + thickness_b * n1 ) / window, 0.0, 1.0 );
        EmitVertex();

        // end at positive normal
        gl_Position = vec4( ( p2 + thickness_b * n2 ) / window, 0.0, 1.0 );
        EmitVertex();
    }
    else {
        // proceed to negative normal
        gl_Position = vec4( ( p2 - thickness_b * n1 ) / window, 0.0, 1.0 );
        EmitVertex();

        // proceed to positive miter
        gl_Position = vec4( ( p2 + length_b * miter_b ) / window, 0.0, 1.0 );
        EmitVertex();

        // end at negative normal
        gl_Position = vec4( ( p2 - thickness_b * n2 ) / window, 0.0, 1.0 );
        EmitVertex();
    }

    EndPrimitive();
}
