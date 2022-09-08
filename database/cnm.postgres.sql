--
-- PostgreSQL database dump
--

-- Dumped from database version 14.3
-- Dumped by pg_dump version 14.3

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- Name: pgcrypto; Type: EXTENSION; Schema: -; Owner: -
--

CREATE EXTENSION IF NOT EXISTS pgcrypto WITH SCHEMA public;


--
-- Name: EXTENSION pgcrypto; Type: COMMENT; Schema: -; Owner: 
--

COMMENT ON EXTENSION pgcrypto IS 'cryptographic functions';


--
-- Name: cnm_role; Type: TYPE; Schema: public; Owner: cnm
--

CREATE TYPE public.cnm_role AS ENUM (
    'admin',
    'bandadmin',
    'regular'
);


ALTER TYPE public.cnm_role OWNER TO cnm;

--
-- Name: org_status; Type: TYPE; Schema: public; Owner: cnm
--

CREATE TYPE public.org_status AS ENUM (
    'todo',
    'raise',
    'success',
    'failure',
    'pending'
);


ALTER TYPE public.org_status OWNER TO cnm;

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: activity; Type: TABLE; Schema: public; Owner: cnm
--

CREATE TABLE public.activity (
    id integer NOT NULL,
    id_org integer NOT NULL,
    name character varying(128) NOT NULL,
    name_bis character varying(128),
    description character varying(256),
    category character varying(128) DEFAULT 'Non Catégorisé'::character varying NOT NULL,
    city character varying(128),
    postal_code character varying(128),
    creation_stamp timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


ALTER TABLE public.activity OWNER TO cnm;

--
-- Name: activity_id_seq; Type: SEQUENCE; Schema: public; Owner: cnm
--

CREATE SEQUENCE public.activity_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.activity_id_seq OWNER TO cnm;

--
-- Name: activity_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: cnm
--

ALTER SEQUENCE public.activity_id_seq OWNED BY public.activity.id;


--
-- Name: band; Type: TABLE; Schema: public; Owner: cnm
--

CREATE TABLE public.band (
    id integer NOT NULL,
    name character varying(128),
    creation_stamp timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    id_creator integer NOT NULL
);


ALTER TABLE public.band OWNER TO cnm;

--
-- Name: band_id_seq; Type: SEQUENCE; Schema: public; Owner: cnm
--

CREATE SEQUENCE public.band_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.band_id_seq OWNER TO cnm;

--
-- Name: band_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: cnm
--

ALTER SEQUENCE public.band_id_seq OWNED BY public.band.id;


--
-- Name: cnm_user; Type: TABLE; Schema: public; Owner: cnm
--

CREATE TABLE public.cnm_user (
    id integer NOT NULL,
    pseudo character varying(64) NOT NULL,
    name character varying(64) NOT NULL,
    firstname character varying(64) NOT NULL,
    email character varying(128) NOT NULL,
    pwd character varying(60) NOT NULL,
    creation_stamp timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    last_login timestamp without time zone,
    verified boolean DEFAULT false NOT NULL,
    verify_chain character varying(32) DEFAULT md5((random())::text) NOT NULL
);


ALTER TABLE public.cnm_user OWNER TO cnm;

--
-- Name: cnm_user_id_seq; Type: SEQUENCE; Schema: public; Owner: cnm
--

CREATE SEQUENCE public.cnm_user_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.cnm_user_id_seq OWNER TO cnm;

--
-- Name: cnm_user_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: cnm
--

ALTER SEQUENCE public.cnm_user_id_seq OWNED BY public.cnm_user.id;


--
-- Name: contact; Type: TABLE; Schema: public; Owner: cnm
--

CREATE TABLE public.contact (
    id integer NOT NULL,
    id_org integer NOT NULL,
    name character varying(64) NOT NULL,
    firstname character varying(64),
    email character varying(64),
    phone character varying(64),
    address character varying(512),
    zip_code character varying(16),
    city character varying(60),
    creation_stamp timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    id_band integer NOT NULL
);


ALTER TABLE public.contact OWNER TO cnm;

--
-- Name: contact_id_seq; Type: SEQUENCE; Schema: public; Owner: cnm
--

CREATE SEQUENCE public.contact_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.contact_id_seq OWNER TO cnm;

--
-- Name: contact_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: cnm
--

ALTER SEQUENCE public.contact_id_seq OWNED BY public.contact.id;


--
-- Name: note; Type: TABLE; Schema: public; Owner: cnm
--

CREATE TABLE public.note (
    id integer NOT NULL,
    id_activity integer NOT NULL,
    note text NOT NULL,
    creation_stamp timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    id_user integer NOT NULL,
    id_band integer NOT NULL
);


ALTER TABLE public.note OWNER TO cnm;

--
-- Name: note_id_seq; Type: SEQUENCE; Schema: public; Owner: cnm
--

CREATE SEQUENCE public.note_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.note_id_seq OWNER TO cnm;

--
-- Name: note_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: cnm
--

ALTER SEQUENCE public.note_id_seq OWNED BY public.note.id;


--
-- Name: org; Type: TABLE; Schema: public; Owner: cnm
--

CREATE TABLE public.org (
    id integer NOT NULL,
    name character varying(128) NOT NULL,
    name_bis character varying(128),
    description character varying(256),
    creation_stamp timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


ALTER TABLE public.org OWNER TO cnm;

--
-- Name: org_assign; Type: TABLE; Schema: public; Owner: cnm
--

CREATE TABLE public.org_assign (
    id integer NOT NULL,
    id_org integer NOT NULL,
    id_user integer NOT NULL,
    creation_stamp timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    id_band integer NOT NULL,
    status public.org_status DEFAULT 'todo'::public.org_status NOT NULL
);


ALTER TABLE public.org_assign OWNER TO cnm;

--
-- Name: org_assign_id_seq; Type: SEQUENCE; Schema: public; Owner: cnm
--

CREATE SEQUENCE public.org_assign_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.org_assign_id_seq OWNER TO cnm;

--
-- Name: org_assign_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: cnm
--

ALTER SEQUENCE public.org_assign_id_seq OWNED BY public.org_assign.id;


--
-- Name: org_id_seq; Type: SEQUENCE; Schema: public; Owner: cnm
--

CREATE SEQUENCE public.org_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.org_id_seq OWNER TO cnm;

--
-- Name: org_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: cnm
--

ALTER SEQUENCE public.org_id_seq OWNED BY public.org.id;


--
-- Name: user_band; Type: TABLE; Schema: public; Owner: cnm
--

CREATE TABLE public.user_band (
    id_user integer NOT NULL,
    id_band integer NOT NULL,
    is_admin boolean DEFAULT false NOT NULL
);


ALTER TABLE public.user_band OWNER TO cnm;

--
-- Name: activity id; Type: DEFAULT; Schema: public; Owner: cnm
--

ALTER TABLE ONLY public.activity ALTER COLUMN id SET DEFAULT nextval('public.activity_id_seq'::regclass);


--
-- Name: band id; Type: DEFAULT; Schema: public; Owner: cnm
--

ALTER TABLE ONLY public.band ALTER COLUMN id SET DEFAULT nextval('public.band_id_seq'::regclass);


--
-- Name: cnm_user id; Type: DEFAULT; Schema: public; Owner: cnm
--

ALTER TABLE ONLY public.cnm_user ALTER COLUMN id SET DEFAULT nextval('public.cnm_user_id_seq'::regclass);


--
-- Name: contact id; Type: DEFAULT; Schema: public; Owner: cnm
--

ALTER TABLE ONLY public.contact ALTER COLUMN id SET DEFAULT nextval('public.contact_id_seq'::regclass);


--
-- Name: note id; Type: DEFAULT; Schema: public; Owner: cnm
--

ALTER TABLE ONLY public.note ALTER COLUMN id SET DEFAULT nextval('public.note_id_seq'::regclass);


--
-- Name: org id; Type: DEFAULT; Schema: public; Owner: cnm
--

ALTER TABLE ONLY public.org ALTER COLUMN id SET DEFAULT nextval('public.org_id_seq'::regclass);


--
-- Name: org_assign id; Type: DEFAULT; Schema: public; Owner: cnm
--

ALTER TABLE ONLY public.org_assign ALTER COLUMN id SET DEFAULT nextval('public.org_assign_id_seq'::regclass);


--
-- Name: activity activity_pkey; Type: CONSTRAINT; Schema: public; Owner: cnm
--

ALTER TABLE ONLY public.activity
    ADD CONSTRAINT activity_pkey PRIMARY KEY (id);


--
-- Name: band band_pkey; Type: CONSTRAINT; Schema: public; Owner: cnm
--

ALTER TABLE ONLY public.band
    ADD CONSTRAINT band_pkey PRIMARY KEY (id);


--
-- Name: cnm_user cnm_user_email_key; Type: CONSTRAINT; Schema: public; Owner: cnm
--

ALTER TABLE ONLY public.cnm_user
    ADD CONSTRAINT cnm_user_email_key UNIQUE (email);


--
-- Name: cnm_user cnm_user_pkey; Type: CONSTRAINT; Schema: public; Owner: cnm
--

ALTER TABLE ONLY public.cnm_user
    ADD CONSTRAINT cnm_user_pkey PRIMARY KEY (id);


--
-- Name: contact contact_pkey; Type: CONSTRAINT; Schema: public; Owner: cnm
--

ALTER TABLE ONLY public.contact
    ADD CONSTRAINT contact_pkey PRIMARY KEY (id);


--
-- Name: note note_pkey; Type: CONSTRAINT; Schema: public; Owner: cnm
--

ALTER TABLE ONLY public.note
    ADD CONSTRAINT note_pkey PRIMARY KEY (id);


--
-- Name: org_assign org_assign_pkey; Type: CONSTRAINT; Schema: public; Owner: cnm
--

ALTER TABLE ONLY public.org_assign
    ADD CONSTRAINT org_assign_pkey PRIMARY KEY (id);


--
-- Name: org org_pkey; Type: CONSTRAINT; Schema: public; Owner: cnm
--

ALTER TABLE ONLY public.org
    ADD CONSTRAINT org_pkey PRIMARY KEY (id);


--
-- Name: user_band user_band_pkey; Type: CONSTRAINT; Schema: public; Owner: cnm
--

ALTER TABLE ONLY public.user_band
    ADD CONSTRAINT user_band_pkey PRIMARY KEY (id_user, id_band);


--
-- Name: activity activity_id_org_fkey; Type: FK CONSTRAINT; Schema: public; Owner: cnm
--

ALTER TABLE ONLY public.activity
    ADD CONSTRAINT activity_id_org_fkey FOREIGN KEY (id_org) REFERENCES public.org(id);


--
-- Name: band band_id_creator_fkey; Type: FK CONSTRAINT; Schema: public; Owner: cnm
--

ALTER TABLE ONLY public.band
    ADD CONSTRAINT band_id_creator_fkey FOREIGN KEY (id_creator) REFERENCES public.cnm_user(id);


--
-- Name: contact contact_id_band_fkey; Type: FK CONSTRAINT; Schema: public; Owner: cnm
--

ALTER TABLE ONLY public.contact
    ADD CONSTRAINT contact_id_band_fkey FOREIGN KEY (id_band) REFERENCES public.band(id);


--
-- Name: contact contact_id_org_fkey; Type: FK CONSTRAINT; Schema: public; Owner: cnm
--

ALTER TABLE ONLY public.contact
    ADD CONSTRAINT contact_id_org_fkey FOREIGN KEY (id_org) REFERENCES public.org(id);


--
-- Name: note note_id_activity_fkey; Type: FK CONSTRAINT; Schema: public; Owner: cnm
--

ALTER TABLE ONLY public.note
    ADD CONSTRAINT note_id_activity_fkey FOREIGN KEY (id_activity) REFERENCES public.activity(id);


--
-- Name: note note_id_band_fkey; Type: FK CONSTRAINT; Schema: public; Owner: cnm
--

ALTER TABLE ONLY public.note
    ADD CONSTRAINT note_id_band_fkey FOREIGN KEY (id_band) REFERENCES public.band(id);


--
-- Name: note note_id_user_fkey; Type: FK CONSTRAINT; Schema: public; Owner: cnm
--

ALTER TABLE ONLY public.note
    ADD CONSTRAINT note_id_user_fkey FOREIGN KEY (id_user) REFERENCES public.cnm_user(id);


--
-- Name: org_assign org_assign_id_band_fkey; Type: FK CONSTRAINT; Schema: public; Owner: cnm
--

ALTER TABLE ONLY public.org_assign
    ADD CONSTRAINT org_assign_id_band_fkey FOREIGN KEY (id_band) REFERENCES public.band(id);


--
-- Name: org_assign org_assign_id_org_fkey; Type: FK CONSTRAINT; Schema: public; Owner: cnm
--

ALTER TABLE ONLY public.org_assign
    ADD CONSTRAINT org_assign_id_org_fkey FOREIGN KEY (id_org) REFERENCES public.org(id);


--
-- Name: org_assign org_assign_id_user_fkey; Type: FK CONSTRAINT; Schema: public; Owner: cnm
--

ALTER TABLE ONLY public.org_assign
    ADD CONSTRAINT org_assign_id_user_fkey FOREIGN KEY (id_user) REFERENCES public.cnm_user(id);


--
-- Name: user_band user_band_id_band_fkey; Type: FK CONSTRAINT; Schema: public; Owner: cnm
--

ALTER TABLE ONLY public.user_band
    ADD CONSTRAINT user_band_id_band_fkey FOREIGN KEY (id_band) REFERENCES public.band(id);


--
-- Name: user_band user_band_id_user_fkey; Type: FK CONSTRAINT; Schema: public; Owner: cnm
--

ALTER TABLE ONLY public.user_band
    ADD CONSTRAINT user_band_id_user_fkey FOREIGN KEY (id_user) REFERENCES public.cnm_user(id);


--
-- PostgreSQL database dump complete
--

