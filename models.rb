#!/usr/bin/env ruby

require './IDL'
include IDL

newtype 'SuspectId', :elm => 'String', :rust => 'String'

enum 'ServerCommand' do |e|
    e.variant 'Connected'
    e.variant 'BecomeSuspect' do |r|
        r.field 'suspectId', 'SuspectId'
    end
    e.variant 'BecomeInvestigator' do |r|
        r.field 'suspectId', 'SuspectId'
    end
    e.variant 'Echo' do |r|
        r.field 'message', 'String'
    end
end

enum 'ClientCommand' do |e|
    e.variant 'LogInAsSuspect'
    e.variant 'LogInAsInvestigator' do |r|
        r.field 'suspectId', 'SuspectId'
    end
    e.variant 'InvestigatorShout' do |r|
        r.field 'message', 'String'
    end
end
    
File.write("client/src/Commands.elm", elm)
File.write("server/src/commands.rs", rust)
