require 'erb'

Variant = Struct.new('Variant', :name, :payload)

class Enum < Object

    attr_accessor :name
    attr_accessor :variants

    def initialize(name)
        @name = name
        @variants = []
    end

    def variant(name, &fields)
        if fields
            r = Record::new()
            fields.call(r)
        else
            r = nil
        end
        variants << Variant.new(name, r)
    end

    def elm
        "type #{@name}\n    = " + variants.map { |v|
            if v.payload then
                "#{v.name}\n        #{v.payload.elm}"
            else
                "#{v.name}"
            end
        }.join("\n    | ") + "\n"
    end

    def rust
        "enum #{@name} {\n" + "\n}"
    end

end

Field = Struct.new('Field', :name, :type)

class Record < Object

    attr_accessor :fields

    def initialize()
        @fields = []
    end

    def field(name, type)
        fields << Field.new(name, type)
    end

    def elm
        "{ " + fields.map { |f|
            "#{f.name} : #{f.type}"
        }.join("\n         , ") + "\n        }"
    end

end

NewType = Struct.new('NewType', :name, :languages)

module IDL

    $decls = []

    def enum(name, &body)
        e = Enum.new(name)
        body.call(e)
        $decls << e
    end

    def newtype(name, languages = {})
        $decls << NewType.new(name, languages)
    end

    def elm
        decls = $decls
        $elm.result(binding)
    end

    def rust
        decls = $decls
        $rust.result(binding)
    end

end

def type_to_var(s)
    result = s.dup
    result[0] = result[0].downcase
    result
end

def to_underscores(s)
    s.gsub(/(?<=[a-z])([A-Z])/) { '_' + $1.downcase }
end

def elm_encode(decls, t)
    case t
    when 'String'
        'string'
    else
        decls.each do |d|
            if d.instance_of?(NewType) then
                return elm_encode(decls, d.languages[:elm])
            end
        end
        raise "Unknown type: #{t}"
    end
end

$elm = ERB.new(<<EOELM, 0, '%')
module Commands exposing
% decls.each_with_index do |decl, i|
% punct = if i == 0 then '(' else ',' end
% if decl.instance_of?(NewType)
    <%= punct %> <%= decl.name %>
% elsif decl.instance_of?(Enum)
    <%= punct %> <%= decl.name %>(..)
    , encode<%= decl.name %>
    , <%= type_to_var(decl.name) %>Decoder
% end
%end
    )


import Json.Decode exposing (Decoder, int, oneOf, string)
import Json.Decode.Pipeline exposing (decode, required)
import Json.Encode

% decls.each do |decl|
% if decl.instance_of?(NewType)
type alias <%= decl.name %>
    = <%= decl.languages[:elm] %>


% elsif decl.instance_of?(Enum)
type <%= decl.name %>
% decl.variants.each_with_index do |v, i|
    <%= if i == 0 then '=' else '|' end %> <%= v.name %>
% if v.payload then
% v.payload.fields.each_with_index do |f, i|
        <%= if i == 0 then '{' else ',' end %> <%= f.name %> : <%= f.type %>
% end
        }
% end
% end


encode<%= decl.name %> : <%= decl.name %> -> Json.Encode.Value
encode<%= decl.name %> <%= type_to_var(decl.name) %> =
    case <%= type_to_var(decl.name) %> of
% decl.variants.each do |v|
% if v.payload then
        <%= v.name %> <%= type_to_var(v.name) %> ->
            Json.Encode.object
                [ ( "type", Json.Encode.string "<%= v.name %>" )
% v.payload.fields.each do |f|
                , ( "<%= to_underscores(f.name) %>", Json.Encode.<%= elm_encode(decls, f.type) %> <%= type_to_var(v.name) %>.<%= f.name %> )
% end
% else
        <%= v.name %> ->
            Json.Encode.object
                [ ( "type", Json.Encode.string "<%= v.name %>" )
% end
                ]


% end
<%= type_to_var(decl.name) %>FieldDecoder : String -> Decoder <%= decl.name %>
<%= type_to_var(decl.name) %>FieldDecoder t =
    case t of
% decl.variants.each do |v|
        "<%= v.name %>" ->
% if v.payload
            decode (\\<%= v.payload.fields.map { |f| f.name }.join(' ') %> -> <%= v.name %> { <%= v.payload.fields.map { |f| f.name + " = " + f.name }.join(' ') %> })
% v.payload.fields.each do |f|
                |> required "<%= to_underscores(f.name) %>" <%= elm_encode(decls, f.type) %>
% end
% else
            Json.Decode.succeed <%= v.name %>
% end

% end
        _ ->
            Json.Decode.fail "Unexpected type"


<%= type_to_var(decl.name) %>Decoder : Decoder <%= decl.name %>
<%= type_to_var(decl.name) %>Decoder =
    Json.Decode.field "type" string
        |> Json.Decode.andThen <%= type_to_var(decl.name) %>FieldDecoder


% end
% end
EOELM

$rust = ERB.new(<<EORUST, 0, '%>')
% decls.each do |decl|
% if decl.instance_of?(NewType)
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct <%= decl.name %>(pub <%= decl.languages[:rust] %>);

% elsif decl.instance_of?(Enum)
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum <%= decl.name %> {
% decl.variants.each do |v|
    <%= v.name %> <% %>
% if v.payload then
{
% v.payload.fields.each do |f|
        <%= to_underscores(f.name) %>: <%= f.type %>,
% end
    },
% else
,
% end
% end
}

% end
% end
EORUST
